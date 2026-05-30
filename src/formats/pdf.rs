use std::fs::File;
use std::path::Path;

use memmap2::Mmap;

use crate::error::ExtractError;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  pdf_extract::extract_text_from_mem(input).map_err(|err| ExtractError::Parse(format!("pdf: {err}")))
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  crate::input::validate_path_size(path, max_bytes)?;
  let file = File::open(path).map_err(|err| ExtractError::Io(format!("pdf open: {err}")))?;

  let mapped = unsafe { Mmap::map(&file) }.map_err(|err| ExtractError::Io(format!("pdf mmap: {err}")))?;
  pdf_extract::extract_text_from_mem(&mapped).map_err(|err| ExtractError::Parse(format!("pdf: {err}")))
}
