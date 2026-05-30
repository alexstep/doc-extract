use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::error::ExtractError;
use crate::formats::read_path_bytes;

static ESCAPED_HEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\\'[0-9a-fA-F]{2}").unwrap());
static CONTROLS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\\[a-zA-Z]+-?\d* ?").unwrap());
static BRACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[{}]").unwrap());

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  let decoded = super::text::decode_text_with_bom(input);

  let no_hex = ESCAPED_HEX.replace_all(&decoded, " ");
  let no_controls = CONTROLS.replace_all(&no_hex, " ");
  let no_braces = BRACES.replace_all(&no_controls, " ");

  Ok(no_braces.to_string())
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  extract(&read_path_bytes(path, max_bytes)?)
}
