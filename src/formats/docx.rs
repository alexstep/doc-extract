use std::io::{Read, Seek};
use std::path::Path;

use zip::ZipArchive;

use crate::error::ExtractError;
use crate::formats::open_file;
use crate::formats::zip_util::read_zip_entry_limited;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  extract_reader(std::io::Cursor::new(input))
}

pub fn extract_from_path(path: &Path) -> Result<String, ExtractError> {
  extract_reader(open_file(path)?)
}

pub fn extract_reader<R: Read + Seek>(reader: R) -> Result<String, ExtractError> {
  let mut archive =
    ZipArchive::new(reader).map_err(|err| ExtractError::Parse(format!("docx zip: {err}")))?;

  let mut entry = archive
    .by_name("word/document.xml")
    .map_err(|err| ExtractError::Parse(format!("docx word/document.xml: {err}")))?;

  let xml = read_zip_entry_limited(&mut entry, "docx word/document.xml")?;
  super::text::extract_xml(&xml)
}
