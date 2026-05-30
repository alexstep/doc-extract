use std::fs::File;
use std::path::Path;

use crate::error::ExtractError;
use crate::input::read_file_bytes;

pub mod docx;
pub mod epub;
pub mod fb2;
pub mod ical;
pub mod json;
pub mod odt;
pub mod pdf;
pub mod pptx;
pub mod rtf;
pub mod spreadsheet;
pub mod text;
pub mod vcf;

pub fn extract_text(input: &[u8], format: &str) -> Result<String, ExtractError> {
  let format = format.trim().trim_start_matches('.').to_ascii_lowercase();
  let text = extract_inner(&format, input)?;

  let normalized = text.trim().to_string();
  if normalized.is_empty() {
    return Err(ExtractError::EmptyResult);
  }

  Ok(normalized)
}

pub fn extract_text_from_path(path: &Path, format: &str, max_bytes: usize) -> Result<String, ExtractError> {
  let format = format.trim().trim_start_matches('.').to_ascii_lowercase();
  let text = match format.as_str() {
    "pdf" => pdf::extract_from_path(path, max_bytes)?,
    "docx" | "docm" => docx::extract_from_path(path)?,
    "xlsx" | "xls" | "ods" => spreadsheet::extract_from_path(path)?,
    "pptx" | "pptm" => pptx::extract_from_path(path)?,
    "epub" => epub::extract_from_path(path)?,
    "rtf" => rtf::extract_from_path(path, max_bytes)?,
    "odt" => odt::extract_from_path(path)?,
    "fb2" => fb2::extract_from_path(path, max_bytes)?,
    "ics" | "ifb" | "ical" => ical::extract_from_path(path, max_bytes)?,
    "json" => json::extract_from_path(path, max_bytes)?,
    "jsonl" | "ndjson" => json::extract_jsonl_from_path(path, max_bytes)?,
    "vcf" | "vcard" => vcf::extract_from_path(path, max_bytes)?,
    "csv" => text::extract_csv_from_path(path, b',', max_bytes)?,
    "tsv" => text::extract_csv_from_path(path, b'\t', max_bytes)?,
    "html" | "htm" | "xhtml" => text::extract_html_from_path(path, max_bytes)?,
    "xml" => text::extract_xml_from_path(path, max_bytes)?,
    "txt" | "md" | "markdown" | "log" => text::extract_plain_from_path(path, max_bytes)?,
    _ => return Err(ExtractError::UnsupportedFormat(format)),
  };

  let normalized = text.trim().to_string();
  if normalized.is_empty() {
    return Err(ExtractError::EmptyResult);
  }

  Ok(normalized)
}

fn extract_inner(format: &str, bytes: &[u8]) -> Result<String, ExtractError> {
  match format {
    "pdf" => pdf::extract(bytes),
    "docx" | "docm" => docx::extract(bytes),
    "xlsx" | "xls" | "ods" => spreadsheet::extract(bytes),
    "pptx" | "pptm" => pptx::extract(bytes),
    "epub" => epub::extract(bytes),
    "rtf" => rtf::extract(bytes),
    "odt" => odt::extract(bytes),
    "fb2" => fb2::extract(bytes),
    "ics" | "ifb" | "ical" => ical::extract(bytes),
    "json" => json::extract(bytes),
    "jsonl" | "ndjson" => json::extract_jsonl(bytes),
    "vcf" | "vcard" => vcf::extract(bytes),
    "csv" => text::extract_csv(bytes, b','),
    "tsv" => text::extract_csv(bytes, b'\t'),
    "html" | "htm" | "xhtml" => text::extract_html(bytes),
    "xml" => text::extract_xml(bytes),
    "txt" | "md" | "markdown" | "log" => text::extract_plain(bytes),
    other => Err(ExtractError::UnsupportedFormat(other.to_string())),
  }
}

pub(crate) fn open_file(path: &Path) -> Result<File, ExtractError> {
  File::open(path).map_err(|err| ExtractError::Io(format!("open: {err}")))
}

pub(crate) fn read_path_bytes(path: &Path, max_bytes: usize) -> Result<Vec<u8>, ExtractError> {
  read_file_bytes(path, max_bytes)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn alias_ical_matches_ics() {
    let input = include_str!("../../fixtures/sample.ics");
    let ics = extract_text(input.as_bytes(), "ics").unwrap();
    let ical = extract_text(input.as_bytes(), "ical").unwrap();
    assert_eq!(ics, ical);
  }
}
