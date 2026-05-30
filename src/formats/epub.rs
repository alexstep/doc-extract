use std::io::{Cursor, Read, Seek};
use std::path::Path;

use zip::ZipArchive;

use crate::error::ExtractError;
use crate::formats::open_file;
use crate::limits::max_entry_size;

const MAX_TOTAL_EPUB_SIZE: usize = 10 * 1024 * 1024;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  extract_reader(Cursor::new(input))
}

pub fn extract_from_path(path: &Path) -> Result<String, ExtractError> {
  extract_reader(open_file(path)?)
}

pub fn extract_reader<R: Read + Seek>(reader: R) -> Result<String, ExtractError> {
  let mut archive =
    ZipArchive::new(reader).map_err(|err| ExtractError::Parse(format!("epub zip: {err}")))?;
  let mut chunks = Vec::new();
  let mut total_size: usize = 0;

  for index in 0..archive.len() {
    let mut file = archive
      .by_index(index)
      .map_err(|err| ExtractError::Parse(format!("epub entry #{index}: {err}")))?;
    let name = file.name().to_ascii_lowercase();

    if !(name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm")) {
      continue;
    }

    let mut body = Vec::new();
    file
      .take(max_entry_size() as u64)
      .read_to_end(&mut body)
      .map_err(|err| ExtractError::Parse(format!("epub read {name}: {err}")))?;

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
