use std::sync::{LazyLock, RwLock};

use tokio::sync::Semaphore;

use crate::error::ExtractError;

const DEFAULT_MAX_CONCURRENT: usize = 32;
const MAX_LIMIT: usize = 512;

struct ConcurrencyState {
  semaphore: std::sync::Arc<Semaphore>,
}

static STATE: LazyLock<RwLock<ConcurrencyState>> = LazyLock::new(|| {
  RwLock::new(ConcurrencyState {
    semaphore: std::sync::Arc::new(Semaphore::new(initial_limit())),
  })
});

static MAX_WORKING_SET_BYTES: LazyLock<RwLock<Option<usize>>> = LazyLock::new(|| RwLock::new(initial_working_set()));
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

  let limit = (n as usize).clamp(1, MAX_LIMIT);
  let mut state = STATE.write().expect("concurrency state lock");
  state.semaphore = std::sync::Arc::new(Semaphore::new(limit));
}

pub fn set_max_working_set_bytes(n: u32) {
  let limit = if n == 0 {
    None
  } else {
    Some((n as usize).saturating_mul(1024 * 1024))
  };
  *MAX_WORKING_SET_BYTES
    .write()
    .expect("working set lock") = limit;
}

async fn acquire_working_set(weight: usize) -> Result<WorkingSetGuard, ExtractError> {
  let limit = *MAX_WORKING_SET_BYTES.read().expect("working set lock");
  let Some(limit) = limit else {
    return Ok(WorkingSetGuard { weight: 0 });
  };

  let weight = weight.max(1);
  loop {
    {
      let mut in_use = WORKING_SET_IN_USE.write().expect("working set in use lock");
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
    let mut in_use = WORKING_SET_IN_USE.write().expect("working set in use lock");
    *in_use = in_use.saturating_sub(self.weight);
  }
}

pub async fn with_permit<T, F>(weight_bytes: usize, f: F) -> Result<T, ExtractError>
where
  F: FnOnce() -> Result<T, ExtractError> + Send + 'static,
  T: Send + 'static,
{
  let semaphore = {
    let state = STATE.read().expect("concurrency state lock");
    state.semaphore.clone()
  };

  let _permit = semaphore
    .acquire()
    .await
    .map_err(|_| ExtractError::TaskJoin)?;

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
}
