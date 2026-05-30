use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use zip::ZipArchive;

use crate::error::ExtractError;
use crate::formats::text::decode_text_with_bom;
use crate::formats::zip_util::read_zip_entry_limited;
use crate::input::validate_path_size;
use crate::limits::{MAX_IMAGE_SIZE, effective_entry_size};

mod format_for_ai;

const PASSKIT_PASS_TYPES: [&str; 5] = ["eventTicket", "boardingPass", "coupon", "generic", "storeCard"];
const STRIP_IMAGE_PRIORITY: [&str; 3] = ["strip@2x.png", "strip@3x.png", "strip.png"];

#[derive(Debug, Clone)]
pub struct ParsedPkPass {
  pub pass: Value,
  pub localization: Option<String>,
  pub strip_image: Option<String>,
}

pub fn parse_pkpass(input: &[u8], max_bytes: usize) -> Result<Option<ParsedPkPass>, ExtractError> {
  let entry_limit = effective_entry_size(max_bytes);
  let files = match read_relevant_files(Cursor::new(input), entry_limit) {
    Ok(files) => files,
    Err(_) => return Ok(None),
  };
  build_parsed_pkpass(files)
}

pub fn parse_pkpass_from_path(path: &Path, max_bytes: usize) -> Result<Option<ParsedPkPass>, ExtractError> {
  validate_path_size(path, max_bytes)?;
  let entry_limit = effective_entry_size(max_bytes);
  let file = File::open(path).map_err(|err| ExtractError::Io(format!("pkpass open: {err}")))?;
  let files = match read_relevant_files(file, entry_limit) {
    Ok(files) => files,
    Err(_) => return Ok(None),
  };
  build_parsed_pkpass(files)
}

pub fn extract_text(input: &[u8], max_bytes: usize) -> Result<Option<String>, ExtractError> {
  let parsed = parse_pkpass(input, max_bytes)?;
  format_parsed(parsed)
}

pub fn extract_text_from_path(path: &Path, max_bytes: usize) -> Result<Option<String>, ExtractError> {
  let parsed = parse_pkpass_from_path(path, max_bytes)?;
  format_parsed(parsed)
}

fn format_parsed(parsed: Option<ParsedPkPass>) -> Result<Option<String>, ExtractError> {
  let Some(parsed) = parsed else {
    return Ok(None);
  };

  let pretty_json = serde_json::to_string_pretty(&parsed.pass)
    .map_err(|err| ExtractError::Parse(format!("pkpass pass.json stringify: {err}")))?;
  let event_info = format_for_ai::format_event_info(&pretty_json, parsed.localization.as_deref());
  Ok(Some(event_info))
}

fn build_parsed_pkpass(files: HashMap<String, Vec<u8>>) -> Result<Option<ParsedPkPass>, ExtractError> {
  let pass_bytes = match files.get("pass.json") {
    Some(bytes) => bytes,
    None => return Ok(None),
  };

  let pass_json: Value = match serde_json::from_slice(pass_bytes) {
    Ok(value) => value,
    Err(_) => return Ok(None),
  };

  if !is_valid_pass(&pass_json) {
    return Ok(None);
  }

  let localization = largest_strings_file(&files).map(|bytes| decode_text_with_bom(bytes));
  let strip_image = read_strip_image(&files);

  Ok(Some(ParsedPkPass {
    pass: pass_json,
    localization,
    strip_image,
  }))
}

fn read_strip_image(files: &HashMap<String, Vec<u8>>) -> Option<String> {
  for file_name in STRIP_IMAGE_PRIORITY {
    if let Some(image) = files.get(file_name) {
      if image.len() <= MAX_IMAGE_SIZE {
        return Some(format!("data:image/png;base64,{}", STANDARD.encode(image)));
      }
    }
  }
  None
}

fn read_relevant_files<R: Read + Seek>(
  reader: R,
  _entry_limit: usize,
) -> Result<HashMap<String, Vec<u8>>, ExtractError> {
  let mut archive =
    ZipArchive::new(reader).map_err(|err| ExtractError::Parse(format!("pkpass zip: {err}")))?;
  let mut files = HashMap::new();

  for index in 0..archive.len() {
    let mut file = archive
      .by_index(index)
      .map_err(|err| ExtractError::Parse(format!("pkpass entry #{index}: {err}")))?;
    if file.is_dir() {
      continue;
    }
    let name = file.name().to_string();
    if !is_needed_file(&name) {
      continue;
    }

    let bytes = read_zip_entry_limited(&mut file, &format!("pkpass read {name}"))?;
    files.insert(name, bytes);
  }

  Ok(files)
}

fn is_needed_file(name: &str) -> bool {
  if name.contains('\\') || name.contains("..") {
    return false;
  }
  if name == "pass.json" {
    return true;
  }
  if STRIP_IMAGE_PRIORITY.contains(&name) {
    return true;
  }
  is_root_pass_strings(name)
}

fn is_root_pass_strings(name: &str) -> bool {
  const SUFFIX: &str = ".lproj/pass.strings";
  match name.strip_suffix(SUFFIX) {
    Some(prefix) => !prefix.is_empty() && !prefix.contains('/'),
    None => false,
  }
}

fn largest_strings_file(files: &HashMap<String, Vec<u8>>) -> Option<&[u8]> {
  files
    .iter()
    .filter(|(name, _)| is_root_pass_strings(name))
    .map(|(_, bytes)| bytes.as_slice())
    .max_by_key(|bytes| bytes.len())
}

fn is_valid_pass(pass_json: &Value) -> bool {
  if !pass_json.is_object() {
    return false;
  }
  if pass_json.get("formatVersion").and_then(Value::as_i64) != Some(1) {
    return false;
  }
  PASSKIT_PASS_TYPES
    .iter()
    .any(|field| pass_json.get(*field).map(Value::is_object).unwrap_or(false))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn needed_file_matches_js_root_only_rules() {
    assert!(is_needed_file("pass.json"));
    assert!(is_needed_file("en.lproj/pass.strings"));
    assert!(is_needed_file("strip.png"));
    assert!(is_needed_file("strip@2x.png"));
    assert!(!is_needed_file("nested/en.lproj/pass.strings"));
    assert!(!is_needed_file("subdir/pass.json"));
    assert!(!is_needed_file("../pass.json"));
  }
}
