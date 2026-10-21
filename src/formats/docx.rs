use std::io::{Cursor, Read, Seek};
use std::path::Path;

use crate::error::ExtractError;
use crate::formats::open_file;
use crate::formats::zip_util::{open_zip, InflateBudget};

pub fn extract(input: &[u8], max_bytes: usize) -> Result<String, ExtractError> {
  extract_reader(Cursor::new(input), max_bytes)
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  extract_reader(open_file(path)?, max_bytes)
}

pub fn extract_reader<R: Read + Seek>(reader: R, max_bytes: usize) -> Result<String, ExtractError> {
  let mut archive = open_zip(reader, "docx")?;
  let mut budget = InflateBudget::new(max_bytes);

  let mut entry = archive
    .by_name("word/document.xml")
    .map_err(|err| ExtractError::Parse(format!("docx word/document.xml: {err}")))?;

  let xml = budget.read_entry(&mut entry, "docx word/document.xml")?;
  super::text::extract_xml(&xml)
}
