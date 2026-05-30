use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::path::Path;

use ical::IcalParser;

use crate::error::ExtractError;
use crate::input::validate_path_size;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  match extract_reader(BufReader::new(Cursor::new(input))) {
    Ok(text) => Ok(text),
    Err(ExtractError::EmptyResult) => Ok(super::text::decode_text_with_bom(input)),
    Err(err) => Err(err),
  }
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  validate_path_size(path, max_bytes)?;
  let file = File::open(path).map_err(|err| ExtractError::Io(format!("ics open: {err}")))?;
  match extract_reader(BufReader::new(file)) {
    Ok(text) => Ok(text),
    Err(ExtractError::EmptyResult) => super::text::extract_plain_from_path(path, max_bytes),
    Err(err) => Err(err),
  }
}

fn extract_reader<R: BufRead>(reader: R) -> Result<String, ExtractError> {
  let mut parser = IcalParser::new(reader);
  let mut out = Vec::new();

  while let Some(item) = parser.next() {
    let calendar = item.map_err(|err| ExtractError::Parse(format!("ical: {err}")))?;
    for event in calendar.events {
      let mut summary = None;
      let mut starts = None;
      let mut ends = None;
      let mut location = None;
      let mut description = None;

      for prop in event.properties {
        let name = prop.name.to_ascii_uppercase();
        let value = prop.value.unwrap_or_default();
        match name.as_str() {
          "SUMMARY" => summary = Some(value),
          "DTSTART" => starts = Some(value),
          "DTEND" => ends = Some(value),
          "LOCATION" => location = Some(value),
          "DESCRIPTION" => description = Some(value),
          _ => {}
        }
      }

      let mut lines = Vec::new();
      if let Some(v) = summary {
        lines.push(format!("Summary: {v}"));
      }
      if let Some(v) = starts {
        lines.push(format!("Starts: {v}"));
      }
      if let Some(v) = ends {
        lines.push(format!("Ends: {v}"));
      }
      if let Some(v) = location {
        lines.push(format!("Location: {v}"));
      }
      if let Some(v) = description {
        lines.push(format!("Description: {v}"));
      }
      if !lines.is_empty() {
        out.push(lines.join("\n"));
      }
    }
  }

  if out.is_empty() {
    return Err(ExtractError::EmptyResult);
  }

  Ok(out.join("\n\n"))
}
