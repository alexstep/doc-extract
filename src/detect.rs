use std::io::{Cursor, Read, Seek};
use std::path::Path;

use zip::ZipArchive;

use crate::error::ExtractError;
use crate::formats;
use crate::input::{read_file_head, with_file_reader};

const SUPPORTED: &[&str] = &[
  "pdf", "docx", "docm", "xlsx", "xls", "ods", "pptx", "pptm", "epub", "rtf", "odt", "fb2", "ics", "ifb", "ical",
  "json", "jsonl", "ndjson", "vcf", "vcard", "csv", "tsv", "html", "htm", "xhtml", "xml", "txt", "md", "markdown",
  "log", "pkpass",
];

const DETECT_HEAD_BYTES: usize = 4096;

pub fn detect_format(bytes: &[u8], hint: Option<&str>) -> Result<String, ExtractError> {
  let zip_kind = if looks_like_zip(bytes) {
    inspect_zip(Cursor::new(bytes))?
  } else {
    None
  };
  resolve_format(hint, bytes, zip_kind.as_deref())
}

pub fn detect_format_path(path: &Path, hint: Option<&str>) -> Result<String, ExtractError> {
  let head = read_file_head(path, DETECT_HEAD_BYTES)?;
  let zip_kind = if looks_like_zip(&head) {
    with_file_reader(path, |file| inspect_zip(file))?
  } else {
    None
  };
  resolve_format(hint, &head, zip_kind.as_deref())
}

fn resolve_format(hint: Option<&str>, bytes: &[u8], zip_kind: Option<&str>) -> Result<String, ExtractError> {
  if let Some(hint) = hint.filter(|value| !value.trim().is_empty()) {
    let normalized = normalize_hint(hint);
    if !is_supported(&normalized) {
      return Err(ExtractError::UnsupportedFormat(normalized));
    }
    if normalized == "pkpass" {
      return Ok("pkpass".to_string());
    }
    if let Some(zip) = zip_kind {
      if zip_based_format(&normalized) && normalized != zip {
        return Ok(zip.to_string());
      }
    }
    return Ok(normalized);
  }

  if let Some(detected) = detect_from_magic(bytes, zip_kind) {
    return Ok(detected);
  }

  Err(ExtractError::UnsupportedFormat(
    hint.unwrap_or("unknown").to_string(),
  ))
}

fn normalize_hint(hint: &str) -> String {
  match hint.trim().trim_start_matches('.').to_ascii_lowercase().as_str() {
    "vcard" => "vcf".to_string(),
    "ical" | "ifb" => "ics".to_string(),
    "ndjson" => "jsonl".to_string(),
    "htm" | "xhtml" => "html".to_string(),
    "markdown" => "md".to_string(),
    other => other.to_string(),
  }
}

fn is_supported(format: &str) -> bool {
  SUPPORTED.contains(&format)
}

fn zip_based_format(format: &str) -> bool {
  matches!(
    format,
    "docx" | "docm" | "xlsx" | "xls" | "ods" | "pptx" | "pptm" | "epub" | "odt" | "pkpass"
  )
}

fn looks_like_zip(bytes: &[u8]) -> bool {
  bytes.len() >= 4 && bytes[0] == b'P' && bytes[1] == b'K'
}

fn detect_from_magic(bytes: &[u8], zip_kind: Option<&str>) -> Option<String> {
  if bytes.starts_with(b"%PDF") {
    return Some("pdf".to_string());
  }
  if bytes.starts_with(b"{\\rtf") {
    return Some("rtf".to_string());
  }
  let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]);
  let trimmed = head.trim_start();
  if trimmed.starts_with("BEGIN:VCALENDAR") {
    return Some("ics".to_string());
  }
  if trimmed.starts_with("BEGIN:VCARD") {
    return Some("vcf".to_string());
  }
  if let Some(kind) = zip_kind {
    return Some(kind.to_string());
  }
  if trimmed.starts_with('<') {
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("<html") {
      return Some("html".to_string());
    }
    return Some("xml".to_string());
  }
  if trimmed.starts_with('{') || trimmed.starts_with('[') {
    if bytes.contains(&b'\n') {
      let first = trimmed.lines().next().unwrap_or("").trim();
      if first.starts_with('{') {
        return Some("jsonl".to_string());
      }
    }
    return Some("json".to_string());
  }
  if !bytes.is_empty() && bytes.iter().all(|b| b.is_ascii() || b.is_ascii_whitespace()) {
    return Some("txt".to_string());
  }
  None
}

fn inspect_zip<R: Read + Seek>(reader: R) -> Result<Option<String>, ExtractError> {
  let mut archive =
    ZipArchive::new(reader).map_err(|err| ExtractError::Parse(format!("zip detect: {err}")))?;
  let mut names = Vec::new();
  for index in 0..archive.len() {
    let file = archive
      .by_index(index)
      .map_err(|err| ExtractError::Parse(format!("zip detect entry #{index}: {err}")))?;
    if !file.is_dir() {
      names.push(file.name().to_string());
    }
  }

  if names.iter().any(|name| name == "pass.json") {
    return Ok(Some("pkpass".to_string()));
  }
  if names.iter().any(|name| name == "word/document.xml") {
    return Ok(Some("docx".to_string()));
  }
  if names.iter().any(|name| name == "xl/workbook.xml") {
    return Ok(Some("xlsx".to_string()));
  }
  if names.iter().any(|name| name == "ppt/presentation.xml") {
    return Ok(Some("pptx".to_string()));
  }

  if let Ok(mut file) = archive.by_name("mimetype") {
    let mut mimetype = String::new();
    file
      .take(256)
      .read_to_string(&mut mimetype)
      .map_err(|err| ExtractError::Parse(format!("zip mimetype: {err}")))?;
    let mimetype = mimetype.trim();
    if mimetype.contains("epub") {
      return Ok(Some("epub".to_string()));
    }
    if mimetype.contains("opendocument.text") {
      return Ok(Some("odt".to_string()));
    }
    if mimetype.contains("opendocument.spreadsheet") {
      return Ok(Some("ods".to_string()));
    }
  }

  Ok(None)
}

#[allow(dead_code)]
pub fn extract_auto(bytes: &[u8], hint: Option<&str>) -> Result<String, ExtractError> {
  let format = detect_format(bytes, hint)?;
  extract_with_format(bytes, &format)
}

pub fn extract_with_format(bytes: &[u8], format: &str) -> Result<String, ExtractError> {
  let format = normalize_hint(format);
  match format.as_str() {
    "pkpass" => crate::pkpass::extract_text(bytes, crate::limits::global_max_bytes())?
      .ok_or(ExtractError::EmptyResult),
    _ => formats::extract_text(bytes, &format),
  }
}

pub fn extract_with_format_path(path: &Path, format: &str, max_bytes: usize) -> Result<String, ExtractError> {
  let format = normalize_hint(format);
  match format.as_str() {
    "pkpass" => crate::pkpass::extract_text_from_path(path, max_bytes)?
      .ok_or(ExtractError::EmptyResult),
    _ => formats::extract_text_from_path(path, &format, max_bytes),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_pdf_magic() {
    assert_eq!(detect_format(b"%PDF-1.4 test", None).unwrap(), "pdf");
  }

  #[test]
  fn hint_normalizes_vcard() {
    assert_eq!(
      detect_format(b"BEGIN:VCARD\nVERSION:3.0\nEND:VCARD", Some("vcard")).unwrap(),
      "vcf"
    );
  }

  #[test]
  fn detects_plain_text_fallback() {
    assert_eq!(detect_format(b"hello calendar", None).unwrap(), "txt");
  }
}
