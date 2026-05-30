use std::path::Path;

use crate::error::ExtractError;
use crate::formats::read_path_bytes;

const USEFUL_KEYS: [&str; 10] = ["FN", "N", "NICKNAME", "TEL", "EMAIL", "BDAY", "ADR", "ORG", "TITLE", "NOTE"];

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  let raw = super::text::decode_text_with_bom(input);
  let mut cards = Vec::new();

  for block in raw.split("BEGIN:VCARD") {
    if !block.contains("END:VCARD") {
      continue;
    }

    let mut lines = Vec::new();
    let mut pending_key: Option<String> = None;
    let mut pending_value = String::new();

    for line in block.lines() {
      if line.starts_with(' ') || line.starts_with('\t') {
        if pending_key.is_some() {
          pending_value.push_str(line.trim_start());
        }
        continue;
      }

      if let Some(key) = pending_key.take() {
        push_vcard_line(&mut lines, &key, &pending_value);
        pending_value.clear();
      }

      let line = line.trim();
      if line.is_empty() || line == "END:VCARD" {
        continue;
      }

      let Some((key, value)) = line.split_once(':') else {
        continue;
      };
      let key = key.split(';').next().unwrap_or(key).to_ascii_uppercase();
      pending_key = Some(key);
      pending_value = value.to_string();
    }

    if let Some(key) = pending_key.take() {
      push_vcard_line(&mut lines, &key, &pending_value);
    }

    if !lines.is_empty() {
      cards.push(lines.join("\n"));
    }
  }

  if cards.is_empty() {
    return Err(ExtractError::EmptyResult);
  }

  Ok(cards.join("\n\n"))
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  extract(&read_path_bytes(path, max_bytes)?)
}

fn push_vcard_line(lines: &mut Vec<String>, key: &str, value: &str) {
  let value = value.trim();
  if value.is_empty() {
    return;
  }

  if USEFUL_KEYS.contains(&key) {
    lines.push(format!("{key}: {value}"));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_contact_fields() {
    let vcf = "BEGIN:VCARD\nVERSION:3.0\nFN:Alice Example\nBDAY:1990-05-01\nEND:VCARD\n";
    let text = extract(vcf.as_bytes()).unwrap();
    assert!(text.contains("FN: Alice Example"));
    assert!(text.contains("BDAY: 1990-05-01"));
  }
}
