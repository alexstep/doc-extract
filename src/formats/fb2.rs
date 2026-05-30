use std::path::Path;

use crate::error::ExtractError;
use crate::formats::read_path_bytes;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  super::text::extract_xml(input).map_err(|err| match err {
    ExtractError::EmptyResult => ExtractError::Parse("fb2: no text found".into()),
    other => other,
  })
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  extract(&read_path_bytes(path, max_bytes)?)
}
