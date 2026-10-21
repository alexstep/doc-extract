use std::path::Path;

use crate::error::ExtractError;
use crate::input::read_file_bytes;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  pdf_extract::extract_text_from_mem(input)
    .map_err(|err| ExtractError::Parse(format!("pdf: {err}")))
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  let bytes = read_file_bytes(path, max_bytes)?;
  extract(&bytes)
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::*;

  #[test]
  fn path_matches_memory_without_mapping_the_file() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/sample1.pdf");
    let bytes = std::fs::read(&path).unwrap();
    let from_memory = extract(&bytes).unwrap();
    let from_path = extract_from_path(&path, usize::MAX).unwrap();
    assert_eq!(from_memory, from_path);
    assert!(from_memory.contains("Sample PDF"));
  }
}
