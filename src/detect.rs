pub mod text_heuristic;

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

const TEXT_EXTENSION_HINTS: &[&str] = &["txt", "md", "markdown", "csv", "tsv", "log", "json", "jsonl", "html", "xml"];

const MAGIC_HEAD_BYTES: usize = 4096;
const TEXT_HEURISTIC_SAMPLE_BYTES: usize = 32 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownPolicy {
  Reject,
  TextIfLikely,
  TextLossy,
}

impl UnknownPolicy {
  pub fn parse(value: Option<&str>) -> Self {
    match value.map(str::trim).map(|s| s.to_ascii_lowercase()).as_deref() {
      Some("reject") => Self::Reject,
      Some("text-lossy") => Self::TextLossy,
      _ => Self::TextIfLikely,
    }
  }
}

pub struct DetectOptions<'a> {
  pub explicit_format: Option<&'a str>,
  pub extension_hint: Option<&'a str>,
  pub unknown_policy: UnknownPolicy,
}

impl<'a> DetectOptions<'a> {
  pub fn legacy_hint(hint: Option<&'a str>) -> Self {
    Self {
      explicit_format: hint,
      extension_hint: None,
      unknown_policy: UnknownPolicy::TextIfLikely,
    }
  }
}

pub fn detect_format(bytes: &[u8], options: DetectOptions<'_>) -> Result<String, ExtractError> {
  let zip_kind = if looks_like_zip(bytes) {
    inspect_zip(Cursor::new(bytes))?
  } else {
    None
  };
  resolve_format(bytes, zip_kind.as_deref(), options)
}

pub fn detect_format_path(path: &Path, options: DetectOptions<'_>) -> Result<String, ExtractError> {
  let sample = read_file_head(path, TEXT_HEURISTIC_SAMPLE_BYTES)?;
  let zip_kind = if looks_like_zip(&sample[..sample.len().min(MAGIC_HEAD_BYTES)]) {
    with_file_reader(path, |file| inspect_zip(file))?
  } else {
    None
  };
  resolve_format(&sample, zip_kind.as_deref(), options)
}

fn resolve_format(
  bytes: &[u8],
  zip_kind: Option<&str>,
  options: DetectOptions<'_>,
) -> Result<String, ExtractError> {
  let magic = detect_from_magic(bytes, zip_kind);

  if let Some(explicit) = options.explicit_format.filter(|value| !value.trim().is_empty()) {
    return resolve_with_hint(normalize_hint(explicit), bytes, magic.as_deref(), zip_kind, true);
  }

  if let Some(detected) = magic {
    return Ok(detected);
  }

  if let Some(extension) = options.extension_hint.filter(|value| !value.trim().is_empty()) {
    let normalized = normalize_hint(extension);
    if normalized == "pdf" && looks_like_zip(bytes) {
      return apply_unknown_policy(bytes, options.unknown_policy);
    }
    if is_text_extension_hint(&normalized) && !text_heuristic::looks_like_text(bytes) {
      return apply_unknown_policy(bytes, options.unknown_policy);
    }
    return resolve_with_hint(normalized, bytes, None, zip_kind, false);
  }

  apply_unknown_policy(bytes, options.unknown_policy)
}

fn resolve_with_hint(
  normalized: String,
  bytes: &[u8],
  magic: Option<&str>,
  zip_kind: Option<&str>,
  explicit: bool,
) -> Result<String, ExtractError> {
  if !is_supported(&normalized) {
    return Err(ExtractError::UnsupportedFormat(normalized));
  }

  if !explicit {
    if let Some(detected) = magic {
      if magic_conflicts_with_hint(&normalized, Some(detected), zip_kind) {
        return Ok(detected.to_string());
      }
    }
  } else if normalized == "pkpass" {
    return Ok("pkpass".to_string());
  } else if let Some(zip) = zip_kind {
    if zip_based_format(&normalized) && normalized != zip {
      return Ok(zip.to_string());
    }
  }

  if is_text_extension_hint(&normalized) && !text_heuristic::looks_like_text(bytes) {
    return Err(ExtractError::UnsupportedFormat("looks_binary".to_string()));
  }

  Ok(normalized)
}

fn magic_conflicts_with_hint(hint: &str, magic: Option<&str>, zip_kind: Option<&str>) -> bool {
  let Some(magic) = magic else {
    return false;
  };
  if hint == magic {
    return false;
  }
  if zip_kind == Some(magic) && zip_based_format(hint) {
    return hint != magic;
  }
  if hint == "pdf" && magic != "pdf" {
    return true;
  }
  if zip_based_format(hint) && zip_kind == Some(magic) && hint != magic {
    return true;
  }
  hint != magic && !is_text_extension_hint(hint)
}

fn is_text_extension_hint(format: &str) -> bool {
  TEXT_EXTENSION_HINTS.contains(&format)
}

fn apply_unknown_policy(bytes: &[u8], policy: UnknownPolicy) -> Result<String, ExtractError> {
  match policy {
    UnknownPolicy::Reject => Err(ExtractError::UnsupportedFormat("unknown".to_string())),
    UnknownPolicy::TextIfLikely => {
      if text_heuristic::looks_like_text(bytes) {
        Ok("txt".to_string())
      } else {
        Err(ExtractError::UnsupportedFormat("looks_binary".to_string()))
      }
    }
    UnknownPolicy::TextLossy => {
      if text_heuristic::looks_obviously_binary(bytes) {
        Err(ExtractError::UnsupportedFormat("looks_binary".to_string()))
      } else {
        Ok("txt".to_string())
      }
    }
  }
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

  if let Ok(file) = archive.by_name("mimetype") {
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
  let format = detect_format(bytes, DetectOptions::legacy_hint(hint))?;
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
    assert_eq!(
      detect_format(b"%PDF-1.4 test", DetectOptions::legacy_hint(None)).unwrap(),
      "pdf"
    );
  }

  #[test]
  fn hint_normalizes_vcard() {
    assert_eq!(
      detect_format(
        b"BEGIN:VCARD\nVERSION:3.0\nEND:VCARD",
        DetectOptions::legacy_hint(Some("vcard"))
      )
      .unwrap(),
      "vcf"
    );
  }

  #[test]
  fn detects_plain_text_fallback() {
    assert_eq!(
      detect_format(b"hello calendar", DetectOptions::legacy_hint(None)).unwrap(),
      "txt"
    );
  }

  #[test]
  fn rejects_binary_with_reject_policy() {
    let bytes: Vec<u8> = (0..512).map(|i| (i % 256) as u8).collect();
    let result = detect_format(
      &bytes,
      DetectOptions {
        explicit_format: None,
        extension_hint: None,
        unknown_policy: UnknownPolicy::Reject,
      },
    );
    assert!(result.is_err());
  }

  #[test]
  fn magic_overrides_lying_pdf_extension() {
    let zip_head = b"PK\x03\x04";
    let result = detect_format(
      zip_head,
      DetectOptions {
        explicit_format: None,
        extension_hint: Some("pdf"),
        unknown_policy: UnknownPolicy::TextIfLikely,
      },
    );
    assert!(result.is_err() || result.unwrap() != "pdf");
  }

  #[test]
  fn magic_ics_before_txt_extension_hint() {
    let bytes = b"BEGIN:VCALENDAR\nVERSION:2.0\nEND:VCALENDAR";
    let result = detect_format(
      bytes,
      DetectOptions {
        explicit_format: None,
        extension_hint: Some("txt"),
        unknown_policy: UnknownPolicy::TextIfLikely,
      },
    )
    .unwrap();
    assert_eq!(result, "ics");
  }
}
