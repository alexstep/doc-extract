use std::io::{Cursor, Read, Seek};
use std::path::Path;

use crate::error::ExtractError;
use crate::formats::open_file;
use crate::formats::zip_util::{open_zip, zip_entry_name, InflateBudget};

const MAX_TOTAL_EPUB_SIZE: usize = 10 * 1024 * 1024;

pub fn extract(input: &[u8], max_bytes: usize) -> Result<String, ExtractError> {
  extract_reader(Cursor::new(input), max_bytes)
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  extract_reader(open_file(path)?, max_bytes)
}

pub fn extract_reader<R: Read + Seek>(reader: R, max_bytes: usize) -> Result<String, ExtractError> {
  let mut archive = open_zip(reader, "epub")?;
  let mut budget = InflateBudget::new(max_bytes);
  let mut chunks = Vec::new();
  let mut total_size: usize = 0;

  for index in 0..archive.len() {
    let mut file = archive
      .by_index(index)
      .map_err(|err| ExtractError::Parse(format!("epub entry #{index}: {err}")))?;
    let name = zip_entry_name(&file, &format!("epub entry #{index}"))?.to_ascii_lowercase();

    if !(name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm")) {
      continue;
    }

    let body = budget.read_entry(&mut file, &format!("epub read {name}"))?;

    total_size = total_size.saturating_add(body.len());
    if total_size > MAX_TOTAL_EPUB_SIZE {
      break;
    }

    let text = super::text::extract_html(&body)?;
    if !text.trim().is_empty() {
      chunks.push(text);
    }
  }

  Ok(chunks.join("\n\n"))
}
