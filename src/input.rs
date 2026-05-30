use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::error::ExtractError;
use crate::limits::{exceeds_limit, UNLIMITED_BYTES};

pub fn validate_path_size(path: &Path, limit: usize) -> Result<u64, ExtractError> {
  let metadata = fs::metadata(path).map_err(|err| ExtractError::Io(format!("metadata: {err}")))?;
  let size = metadata.len();
  if exceeds_limit(size as usize, limit) {
    return Err(ExtractError::InputTooLarge);
  }
  Ok(size)
}

pub fn read_file_head(path: &Path, max: usize) -> Result<Vec<u8>, ExtractError> {
  let mut file = File::open(path).map_err(|err| ExtractError::Io(format!("open: {err}")))?;
  let mut buf = vec![0_u8; max];
  let read = file
    .read(&mut buf)
    .map_err(|err| ExtractError::Io(format!("read head: {err}")))?;
  buf.truncate(read);
  Ok(buf)
}

pub fn read_file_bytes(path: &Path, limit: usize) -> Result<Vec<u8>, ExtractError> {
  let size = validate_path_size(path, limit)?;
  if size == 0 {
    return Ok(Vec::new());
  }
  let bytes = fs::read(path).map_err(|err| ExtractError::Io(format!("read: {err}")))?;
  if limit != UNLIMITED_BYTES && bytes.len() > limit {
    return Err(ExtractError::InputTooLarge);
  }
  Ok(bytes)
}

pub fn with_file_reader<R, F>(path: &Path, f: F) -> Result<R, ExtractError>
where
  F: FnOnce(&mut File) -> Result<R, ExtractError>,
{
  let mut file = File::open(path).map_err(|err| ExtractError::Io(format!("open: {err}")))?;
  file
    .seek(SeekFrom::Start(0))
    .map_err(|err| ExtractError::Io(format!("seek: {err}")))?;
  f(&mut file)
}
