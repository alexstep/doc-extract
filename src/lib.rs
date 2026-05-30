mod concurrency;
mod detect;
mod error;
mod formats;
mod input;
mod limits;
mod pkpass;

use std::path::PathBuf;

use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use serde_json::Value;

use crate::detect::{detect_format, detect_format_path, extract_with_format, extract_with_format_path};
use crate::error::ExtractError;
use crate::input::validate_path_size;
use crate::limits::{effective_max_bytes, exceeds_limit, global_in_memory_threshold};

#[napi(object)]
pub struct ExtractNativeOptions {
  pub format: Option<String>,
  #[napi(js_name = "maxBytes")]
  pub max_bytes: Option<u32>,
}

#[napi(object)]
pub struct PkPassNativeOptions {
  #[napi(js_name = "maxBytes")]
  pub max_bytes: Option<u32>,
}

#[napi(object)]
pub struct PkPassResult {
  pub pass: Value,
  pub localization: Option<String>,
  #[napi(js_name = "stripImage")]
  pub strip_image: Option<String>,
}

#[napi(js_name = "setMaxConcurrent")]
pub fn set_max_concurrent(n: u32) {
  concurrency::set_max_concurrent(n);
}

#[napi(js_name = "setMaxBytes")]
pub fn set_max_bytes(n: u32) {
  limits::set_max_bytes(n);
}

#[napi(js_name = "setInMemoryThresholdBytes")]
pub fn set_in_memory_threshold_bytes(n: u32) {
  limits::set_in_memory_threshold_bytes(n);
}

#[napi(js_name = "setMaxWorkingSetMB")]
pub fn set_max_working_set_mb(n: u32) {
  concurrency::set_max_working_set_bytes(n);
}

#[napi(js_name = "extractText")]
pub async fn extract_text(input: Buffer, options: Option<ExtractNativeOptions>) -> napi::Result<String> {
  let options = options.unwrap_or(ExtractNativeOptions {
    format: None,
    max_bytes: None,
  });
  let limit = effective_max_bytes(options.max_bytes);
  validate_size(&input, limit)?;
  let bytes = input.to_vec();
  let format_hint = options.format.filter(|value| !value.trim().is_empty());
  let weight = working_set_weight(bytes.len());

  concurrency::with_permit(weight, move || {
    let format = match format_hint.as_deref() {
      Some(format) => detect_format(&bytes, Some(format))?,
      None => detect_format(&bytes, None)?,
    };
    extract_with_format(&bytes, &format)
  })
  .await
  .map_err(napi::Error::from)
}

#[napi(js_name = "extractTextFromPath")]
pub async fn extract_text_from_path(path: String, options: Option<ExtractNativeOptions>) -> napi::Result<String> {
  let options = options.unwrap_or(ExtractNativeOptions {
    format: None,
    max_bytes: None,
  });
  let path_buf = PathBuf::from(path);
  let limit = effective_max_bytes(options.max_bytes);
  let file_size = validate_path_size(&path_buf, limit)? as usize;
  let format_hint = options.format.filter(|value| !value.trim().is_empty());
  let weight = working_set_weight(file_size);

  concurrency::with_permit(weight, move || {
    let format = match format_hint.as_deref() {
      Some(format) => detect_format_path(&path_buf, Some(format))?,
      None => detect_format_path(&path_buf, None)?,
    };
    extract_with_format_path(&path_buf, &format, limit)
  })
  .await
  .map_err(napi::Error::from)
}

#[napi(js_name = "parsePkPass")]
pub async fn parse_pk_pass(input: Buffer, options: Option<PkPassNativeOptions>) -> napi::Result<Option<PkPassResult>> {
  let options = options.unwrap_or(PkPassNativeOptions { max_bytes: None });
  let limit = effective_max_bytes(options.max_bytes);
  validate_size(&input, limit)?;
  let bytes = input.to_vec();
  let weight = working_set_weight(bytes.len());

  let parsed = concurrency::with_permit(weight, move || pkpass::parse_pkpass(&bytes, limit))
    .await
    .map_err(napi::Error::from)?;

  Ok(parsed.map(map_pkpass_result))
}

#[napi(js_name = "parsePkPassFromPath")]
pub async fn parse_pk_pass_from_path(
  path: String,
  options: Option<PkPassNativeOptions>,
) -> napi::Result<Option<PkPassResult>> {
  let options = options.unwrap_or(PkPassNativeOptions { max_bytes: None });
  let path_buf = PathBuf::from(path);
  let limit = effective_max_bytes(options.max_bytes);
  let file_size = validate_path_size(&path_buf, limit)? as usize;
  let weight = working_set_weight(file_size);

  let parsed = concurrency::with_permit(weight, move || pkpass::parse_pkpass_from_path(&path_buf, limit))
    .await
    .map_err(napi::Error::from)?;

  Ok(parsed.map(map_pkpass_result))
}

fn map_pkpass_result(value: pkpass::ParsedPkPass) -> PkPassResult {
  PkPassResult {
    pass: value.pass,
    localization: value.localization,
    strip_image: value.strip_image,
  }
}

fn validate_size(input: &[u8], limit: usize) -> Result<(), ExtractError> {
  if exceeds_limit(input.len(), limit) {
    return Err(ExtractError::InputTooLarge);
  }
  Ok(())
}

fn working_set_weight(file_size: usize) -> usize {
  file_size.min(global_in_memory_threshold()).max(1)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::limits::global_max_bytes;

  #[test]
  fn rejects_large_payload() {
    let limit = global_max_bytes();
    if limit == limits::UNLIMITED_BYTES {
      return;
    }
    let bytes = vec![1_u8; limit + 1];
    let result = validate_size(&bytes, limit);
    assert!(result.is_err());
  }

  #[test]
  fn extracts_plain_text() {
    let text = formats::extract_text("calendar".as_bytes(), "txt");
    assert!(text.is_ok());
    assert_eq!(text.unwrap_or_default(), "calendar");
  }

  #[test]
  fn auto_detects_plain_text() {
    let text = detect::extract_auto("calendar".as_bytes(), None);
    assert!(text.is_ok());
    assert_eq!(text.unwrap_or_default(), "calendar");
  }

  #[test]
  fn extracts_plain_text_from_fixture_path() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/sample.txt");
    let limit = limits::UNLIMITED_BYTES;
    let format = detect::detect_format_path(&path, None).expect("detect path");
    assert_eq!(format, "txt");
    let text = detect::extract_with_format_path(&path, &format, limit).expect("extract path");
    assert!(text.contains("CalendarTG"));
  }
}
