use std::path::Path;

use crate::error::ExtractError;
use crate::formats::read_path_bytes;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  extract_json(input, false)
}

pub fn extract_jsonl(input: &[u8]) -> Result<String, ExtractError> {
  extract_json(input, true)
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  extract(&read_path_bytes(path, max_bytes)?)
}

pub fn extract_jsonl_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  extract_jsonl(&read_path_bytes(path, max_bytes)?)
}

fn extract_json(input: &[u8], jsonl: bool) -> Result<String, ExtractError> {
  let raw = super::text::decode_text_with_bom(input);

  if jsonl {
    let mut blocks = Vec::new();
    for (index, line) in raw.lines().enumerate() {
      let line = line.trim();
      if line.is_empty() {
        continue;
      }
      let value: serde_json::Value =
        serde_json::from_str(line).map_err(|err| ExtractError::Parse(format!("jsonl line {}: {err}", index + 1)))?;
      blocks.push(format!("Record {}:\n{}", index + 1, pretty_value(&value)));
    }
    return Ok(blocks.join("\n\n"));
  }

  let value: serde_json::Value =
    serde_json::from_str(raw.trim()).map_err(|err| ExtractError::Parse(format!("json: {err}")))?;
  Ok(pretty_value(&value))
}

fn pretty_value(value: &serde_json::Value) -> String {
  serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_json_object() {
    let text = extract(br#"{"title":"Demo Event","date":"2026-06-01"}"#).unwrap();
    assert!(text.contains("Demo Event"));
  }

  #[test]
  fn extracts_jsonl() {
    let text = extract_jsonl(b"{\"a\":1}\n{\"b\":2}\n").unwrap();
    assert!(text.contains("Record 1"));
    assert!(text.contains("Record 2"));
  }
}
