use std::sync::{LazyLock, RwLock};

const DEFAULT_MAX_BYTES: usize = 42 * 1024 * 1024;
const DEFAULT_IN_MEMORY_THRESHOLD: usize = 64 * 1024 * 1024;

pub const MAX_ZIP_ENTRY_BYTES: usize = 64 * 1024 * 1024;
pub const UNLIMITED_BYTES: usize = usize::MAX;

static MAX_BYTES: LazyLock<RwLock<usize>> = LazyLock::new(|| RwLock::new(initial_max_bytes()));
static IN_MEMORY_THRESHOLD: LazyLock<RwLock<usize>> =
  LazyLock::new(|| RwLock::new(initial_in_memory_threshold()));

fn initial_max_bytes() -> usize {
  std::env::var("DOCEXTRACT_MAX_BYTES")
    .ok()
    .and_then(|value| value.parse::<usize>().ok())
    .unwrap_or(DEFAULT_MAX_BYTES)
}

fn initial_in_memory_threshold() -> usize {
  std::env::var("DOCEXTRACT_IN_MEMORY_THRESHOLD_MB")
    .ok()
    .and_then(|value| value.parse::<usize>().ok())
    .map(|mb| mb.saturating_mul(1024 * 1024))
    .unwrap_or(DEFAULT_IN_MEMORY_THRESHOLD)
}

pub fn set_max_bytes(n: u32) {
  let limit = if n == 0 { UNLIMITED_BYTES } else { n as usize };
  *MAX_BYTES.write().expect("max bytes lock") = limit;
}

pub fn set_in_memory_threshold_bytes(n: u32) {
  let threshold = if n == 0 {
    DEFAULT_IN_MEMORY_THRESHOLD
  } else {
    (n as usize).saturating_mul(1024 * 1024)
  };
  *IN_MEMORY_THRESHOLD
    .write()
    .expect("in memory threshold lock") = threshold;
}

pub fn global_max_bytes() -> usize {
  *MAX_BYTES.read().expect("max bytes lock")
}

pub fn global_in_memory_threshold() -> usize {
  *IN_MEMORY_THRESHOLD.read().expect("in memory threshold lock")
}

pub fn effective_max_bytes(override_bytes: Option<u32>) -> usize {
  match override_bytes {
    Some(0) => UNLIMITED_BYTES,
    Some(n) if n > 0 => n as usize,
    _ => global_max_bytes(),
  }
}

pub fn exceeds_limit(size: usize, limit: usize) -> bool {
  limit != UNLIMITED_BYTES && size > limit
}

pub fn max_entry_size() -> usize {
  MAX_ZIP_ENTRY_BYTES
}

pub fn effective_entry_size(_max_bytes: usize) -> usize {
  MAX_ZIP_ENTRY_BYTES
}

pub const MAX_IMAGE_SIZE: usize = 1024 * 1024;
pub const MAX_EVENT_INFO_LENGTH: usize = 12_000;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn zero_means_unlimited() {
    assert_eq!(effective_max_bytes(Some(0)), UNLIMITED_BYTES);
  }

  #[test]
  fn exceeds_limit_respects_unlimited() {
    assert!(!exceeds_limit(usize::MAX, UNLIMITED_BYTES));
  }
}
