use std::sync::{LazyLock, RwLock};

use crate::sync::{read, write};

const DEFAULT_MAX_BYTES: usize = 42 * 1024 * 1024;
const DEFAULT_IN_MEMORY_THRESHOLD: usize = 64 * 1024 * 1024;

pub const MAX_ZIP_ENTRY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_ZIP_ENTRIES: usize = 4096;
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
  *write(&MAX_BYTES) = limit;
}

pub fn set_in_memory_threshold_bytes(n: u32) {
  let threshold = if n == 0 {
    DEFAULT_IN_MEMORY_THRESHOLD
  } else {
    (n as usize).saturating_mul(1024 * 1024)
  };
  *write(&IN_MEMORY_THRESHOLD) = threshold;
}

pub fn global_max_bytes() -> usize {
  *read(&MAX_BYTES)
}

pub fn global_in_memory_threshold() -> usize {
  *read(&IN_MEMORY_THRESHOLD)
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

/// Compare a file length without truncating it to `usize`.
///
/// On 32-bit targets `size as usize` drops the high bits, so a file larger
/// than 4 GiB can look smaller than the limit.
pub fn file_len_exceeds(size: u64, limit: usize) -> bool {
  limit != UNLIMITED_BYTES && size > limit as u64
}

pub fn uncompressed_budget(max_bytes: usize) -> usize {
  if max_bytes == UNLIMITED_BYTES {
    MAX_ZIP_ENTRY_BYTES
  } else {
    max_bytes.min(MAX_ZIP_ENTRY_BYTES)
  }
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

  #[test]
  fn file_len_exceeds_keeps_the_high_bits() {
    let size = u64::from(u32::MAX) + 10;
    let truncated = size as u32 as usize;
    assert!(truncated < 100);
    assert!(file_len_exceeds(size, 100));
    assert!(!file_len_exceeds(u64::MAX, UNLIMITED_BYTES));
  }

  #[test]
  fn uncompressed_budget_follows_max_bytes() {
    assert_eq!(uncompressed_budget(1024), 1024);
    assert_eq!(uncompressed_budget(UNLIMITED_BYTES), MAX_ZIP_ENTRY_BYTES);
    assert_eq!(
      uncompressed_budget(MAX_ZIP_ENTRY_BYTES + 1),
      MAX_ZIP_ENTRY_BYTES
    );
  }
}
