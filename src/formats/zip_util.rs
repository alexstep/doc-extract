use std::io::Read;

use zip::read::ZipFile;

use crate::error::ExtractError;
use crate::limits::max_entry_size;

pub fn read_zip_entry_limited(entry: &mut ZipFile<'_>, label: &str) -> Result<Vec<u8>, ExtractError> {
  let max = max_entry_size();
  let mut buf = Vec::new();
  entry
    .take(max as u64 + 1)
    .read_to_end(&mut buf)
    .map_err(|err| ExtractError::Parse(format!("{label} read: {err}")))?;

  if buf.len() > max {
    return Err(ExtractError::InputTooLarge);
  }

  Ok(buf)
}

#[cfg(test)]
mod tests {
  use std::io::{Cursor, Write};

  use zip::write::SimpleFileOptions;
  use zip::ZipWriter;

  use super::*;

  #[test]
  fn rejects_oversized_zip_entry() {
    let mut zip_bytes = Vec::new();
    {
      let mut writer = ZipWriter::new(Cursor::new(&mut zip_bytes));
      let options = SimpleFileOptions::default();
      writer.start_file("big.bin", options).unwrap();
      writer.write_all(&vec![1_u8; max_entry_size() + 1]).unwrap();
      writer.finish().unwrap();
    }

    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes)).unwrap();
    let mut entry = archive.by_index(0).unwrap();
    let result = read_zip_entry_limited(&mut entry, "test");
    assert!(matches!(result, Err(ExtractError::InputTooLarge)));
  }
}
