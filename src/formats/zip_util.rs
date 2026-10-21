use std::io::{Read, Seek, SeekFrom};

use zip::read::ZipFile;
use zip::ZipArchive;

use crate::error::ExtractError;
use crate::limits::{uncompressed_budget, MAX_ZIP_ENTRIES};

pub struct InflateBudget {
  limit: u64,
  used: u64,
}

impl InflateBudget {
  pub fn new(max_bytes: usize) -> Self {
    Self {
      limit: uncompressed_budget(max_bytes) as u64,
      used: 0,
    }
  }

  pub fn read_entry<R: Read + ?Sized>(
    &mut self,
    entry: &mut ZipFile<'_, R>,
    label: &str,
  ) -> Result<Vec<u8>, ExtractError> {
    let room = self.room(entry)?;
    let mut limited = (&mut *entry).take(room.saturating_add(1));
    let mut buf = Vec::new();
    limited
      .read_to_end(&mut buf)
      .map_err(|err| ExtractError::Parse(format!("{label} read: {err}")))?;
    if buf.len() as u64 > room {
      return Err(ExtractError::InputTooLarge);
    }
    self.used = self.used.saturating_add(buf.len() as u64);
    Ok(buf)
  }

  pub fn charge<R: Read + ?Sized>(
    &mut self,
    entry: &mut ZipFile<'_, R>,
    label: &str,
  ) -> Result<(), ExtractError> {
    let room = self.room(entry)?;
    let mut limited = (&mut *entry).take(room.saturating_add(1));
    let mut scratch = [0_u8; 8192];
    let mut got = 0_u64;
    loop {
      let n = limited
        .read(&mut scratch)
        .map_err(|err| ExtractError::Parse(format!("{label} read: {err}")))?;
      if n == 0 {
        break;
      }
      got = got.saturating_add(n as u64);
      if got > room {
        return Err(ExtractError::InputTooLarge);
      }
    }
    self.used = self.used.saturating_add(got);
    Ok(())
  }

  fn room<R: Read + ?Sized>(&self, entry: &ZipFile<'_, R>) -> Result<u64, ExtractError> {
    let declared = entry.size();
    if declared > self.limit {
      return Err(ExtractError::InputTooLarge);
    }
    let room = self.limit.saturating_sub(self.used);
    if declared > room {
      return Err(ExtractError::InputTooLarge);
    }
    Ok(room)
  }
}

pub fn open_zip<R: Read + Seek>(reader: R, label: &str) -> Result<ZipArchive<R>, ExtractError> {
  let archive =
    ZipArchive::new(reader).map_err(|err| ExtractError::Parse(format!("{label} zip: {err}")))?;
  if archive.len() > MAX_ZIP_ENTRIES {
    return Err(ExtractError::InputTooLarge);
  }
  Ok(archive)
}

pub fn zip_entry_name<R: Read + ?Sized>(
  entry: &ZipFile<'_, R>,
  label: &str,
) -> Result<String, ExtractError> {
  entry
    .name()
    .map(|name| name.into_owned())
    .map_err(|err| ExtractError::Parse(format!("{label} name: {err}")))
}

/// Decompress every entry under `max_bytes`, then rewind the reader.
///
/// Spreadsheet parsers open the archive themselves, so the cap has to run
/// before they do.
pub fn enforce_zip_uncompressed_budget<R: Read + Seek>(
  reader: R,
  max_bytes: usize,
) -> Result<R, ExtractError> {
  let mut archive = open_zip(reader, "zip budget")?;
  let mut budget = InflateBudget::new(max_bytes);
  for index in 0..archive.len() {
    let mut entry = archive
      .by_index(index)
      .map_err(|err| ExtractError::Parse(format!("zip budget entry #{index}: {err}")))?;
    if entry.is_dir() {
      continue;
    }
    let label = format!("zip budget entry #{index}");
    budget.charge(&mut entry, &label)?;
  }
  let mut reader = archive.into_inner();
  reader
    .seek(SeekFrom::Start(0))
    .map_err(|err| ExtractError::Io(format!("zip budget rewind: {err}")))?;
  Ok(reader)
}

#[cfg(test)]
mod tests {
  use std::io::{Cursor, Write};

  use zip::write::SimpleFileOptions;
  use zip::ZipWriter;

  use super::*;
  use crate::limits::UNLIMITED_BYTES;

  fn zip_with(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip_bytes = Vec::new();
    {
      let mut writer = ZipWriter::new(Cursor::new(&mut zip_bytes));
      let options = SimpleFileOptions::default();
      for (name, bytes) in files {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
      }
      writer.finish().unwrap();
    }
    zip_bytes
  }

  #[test]
  fn rejects_entry_above_the_call_budget() {
    let payload = vec![1_u8; 2048];
    let zip_bytes = zip_with(&[("word/document.xml", &payload)]);
    let mut archive = open_zip(Cursor::new(zip_bytes), "test").unwrap();
    let mut entry = archive.by_index(0).unwrap();
    let mut budget = InflateBudget::new(1024);
    let result = budget.read_entry(&mut entry, "test");
    assert!(matches!(result, Err(ExtractError::InputTooLarge)));
  }

  #[test]
  fn rejects_when_entries_sum_past_the_budget() {
    let zip_bytes = zip_with(&[("a.xml", &[1_u8; 80]), ("b.xml", &[2_u8; 80])]);
    let result = enforce_zip_uncompressed_budget(Cursor::new(zip_bytes), 100);
    assert!(matches!(result, Err(ExtractError::InputTooLarge)));
  }

  #[test]
  fn accepts_entries_inside_the_budget_and_rewinds() {
    let zip_bytes = zip_with(&[("a.xml", b"hello")]);
    let reader = enforce_zip_uncompressed_budget(Cursor::new(zip_bytes), 1024).unwrap();
    let mut archive = open_zip(reader, "test").unwrap();
    let mut entry = archive.by_index(0).unwrap();
    let mut budget = InflateBudget::new(UNLIMITED_BYTES);
    let body = budget.read_entry(&mut entry, "test").unwrap();
    assert_eq!(body, b"hello");
  }

  #[test]
  fn rejects_too_many_entries() {
    let mut zip_bytes = Vec::new();
    {
      let mut writer = ZipWriter::new(Cursor::new(&mut zip_bytes));
      let options = SimpleFileOptions::default();
      for index in 0..=MAX_ZIP_ENTRIES {
        writer.start_file(format!("f{index}.txt"), options).unwrap();
        writer.write_all(b"x").unwrap();
      }
      writer.finish().unwrap();
    }
    let result = open_zip(Cursor::new(zip_bytes), "test");
    assert!(matches!(result, Err(ExtractError::InputTooLarge)));
  }
}
