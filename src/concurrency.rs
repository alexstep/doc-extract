use std::sync::{Arc, LazyLock, RwLock};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::error::ExtractError;
use crate::sync::{read, write};

const DEFAULT_MAX_CONCURRENT: usize = 32;
const MAX_LIMIT: usize = 512;

struct ConcurrencyState {
  semaphore: Arc<Semaphore>,
  limit: usize,
  /// Permits still held above `limit` after a shrink. Each finished task
  /// discards one until the live cap matches `limit`.
  debt: usize,
}

static STATE: LazyLock<RwLock<ConcurrencyState>> = LazyLock::new(|| {
  let limit = initial_limit();
  RwLock::new(ConcurrencyState {
    semaphore: Arc::new(Semaphore::new(limit)),
    limit,
    debt: 0,
  })
});

static MAX_WORKING_SET_BYTES: LazyLock<RwLock<Option<usize>>> =
  LazyLock::new(|| RwLock::new(initial_working_set()));
static WORKING_SET_IN_USE: LazyLock<RwLock<usize>> = LazyLock::new(|| RwLock::new(0));

fn initial_limit() -> usize {
  std::env::var("DOCEXTRACT_MAX_CONCURRENT")
    .ok()
    .and_then(|value| value.parse::<usize>().ok())
    .unwrap_or(DEFAULT_MAX_CONCURRENT)
    .clamp(1, MAX_LIMIT)
}

fn initial_working_set() -> Option<usize> {
  std::env::var("DOCEXTRACT_MAX_WORKING_SET_MB")
    .ok()
    .and_then(|value| value.parse::<usize>().ok())
    .filter(|value| *value > 0)
    .map(|mb| mb.saturating_mul(1024 * 1024))
}

pub fn set_max_concurrent(n: u32) {
  if n == 0 {
    return;
  }
  let new_limit = (n as usize).clamp(1, MAX_LIMIT);
  let mut state = write(&STATE);
  apply_limit(&mut state, new_limit);
}

fn apply_limit(state: &mut ConcurrencyState, new_limit: usize) {
  if new_limit == state.limit {
    return;
  }
  if new_limit > state.limit {
    let grow = new_limit - state.limit;
    let absorb = grow.min(state.debt);
    state.debt -= absorb;
    let add = grow - absorb;
    if add > 0 {
      state.semaphore.add_permits(add);
    }
  } else {
    let shrink = state.limit - new_limit;
    let forgotten = state.semaphore.forget_permits(shrink);
    state.debt += shrink - forgotten;
  }
  state.limit = new_limit;
}

pub fn set_max_working_set_bytes(n: u32) {
  let limit = if n == 0 {
    None
  } else {
    Some((n as usize).saturating_mul(1024 * 1024))
  };
  *write(&MAX_WORKING_SET_BYTES) = limit;
}

fn ensure_weight_fits(weight: usize) -> Result<(), ExtractError> {
  let limit = *read(&MAX_WORKING_SET_BYTES);
  let Some(limit) = limit else {
    return Ok(());
  };
  if weight.max(1) > limit {
    return Err(ExtractError::WorkingSetExceeded);
  }
  Ok(())
}

async fn acquire_working_set(weight: usize) -> Result<WorkingSetGuard, ExtractError> {
  let limit = *read(&MAX_WORKING_SET_BYTES);
  let Some(limit) = limit else {
    return Ok(WorkingSetGuard { weight: 0 });
  };

  let weight = weight.max(1);
  if weight > limit {
    return Err(ExtractError::WorkingSetExceeded);
  }

  loop {
    {
      let mut in_use = write(&WORKING_SET_IN_USE);
      if in_use.saturating_add(weight) <= limit {
        *in_use += weight;
        return Ok(WorkingSetGuard { weight });
      }
    }
    tokio::task::yield_now().await;
  }
}

struct WorkingSetGuard {
  weight: usize,
}

impl Drop for WorkingSetGuard {
  fn drop(&mut self) {
    if self.weight == 0 {
      return;
    }
    let mut in_use = write(&WORKING_SET_IN_USE);
    *in_use = in_use.saturating_sub(self.weight);
  }
}

struct PermitGuard {
  permit: Option<OwnedSemaphorePermit>,
}

impl Drop for PermitGuard {
  fn drop(&mut self) {
    self.permit.take();
    let mut state = write(&STATE);
    if state.debt > 0 && state.semaphore.forget_permits(1) == 1 {
      state.debt -= 1;
    }
  }
}

pub async fn with_permit<T, F>(weight_bytes: usize, f: F) -> Result<T, ExtractError>
where
  F: FnOnce() -> Result<T, ExtractError> + Send + 'static,
  T: Send + 'static,
{
  ensure_weight_fits(weight_bytes)?;

  let semaphore = {
    let state = read(&STATE);
    Arc::clone(&state.semaphore)
  };

  let permit = semaphore
    .acquire_owned()
    .await
    .map_err(|_| ExtractError::TaskJoin)?;
  let _permit = PermitGuard {
    permit: Some(permit),
  };

  let _working_set = acquire_working_set(weight_bytes).await?;

  tokio::task::spawn_blocking(f)
    .await
    .map_err(|_| ExtractError::TaskJoin)?
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_limit_from_env_or_constant() {
    assert!(initial_limit() >= 1);
    assert!(initial_limit() <= MAX_LIMIT);
  }

  #[test]
  fn zero_leaves_the_limit_unchanged() {
    let previous = read(&STATE).limit;
    set_max_concurrent(4);
    set_max_concurrent(0);
    assert_eq!(read(&STATE).limit, 4);
    set_max_concurrent(previous as u32);
    assert_eq!(read(&STATE).limit, previous);
  }

  #[test]
  fn weight_above_the_working_set_fails_immediately() {
    set_max_working_set_bytes(1);
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap();
    let result = runtime.block_on(acquire_working_set(2 * 1024 * 1024));
    set_max_working_set_bytes(0);
    assert!(matches!(result, Err(ExtractError::WorkingSetExceeded)));
  }
}
