use std::borrow::Cow;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read};
use std::path::Path;

use csv::ReaderBuilder;
use encoding_rs::{UTF_16BE, UTF_16LE, UTF_8, WINDOWS_1252};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::detect::text_heuristic;
use crate::error::ExtractError;
use crate::input::validate_path_size;
use crate::formats::read_path_bytes;

pub fn decode_text_with_bom(input: &[u8]) -> String {
  if input.len() >= 2 && input[0] == 0xFF && input[1] == 0xFE {
    let (text, _, _) = UTF_16LE.decode(&input[2..]);
    return text.into_owned();
  }
  if input.len() >= 2 && input[0] == 0xFE && input[1] == 0xFF {
    let (text, _, _) = UTF_16BE.decode(&input[2..]);
    return text.into_owned();
  }
  if input.len() >= 3 && input[0] == 0xEF && input[1] == 0xBB && input[2] == 0xBF {
    let (text, _, _) = UTF_8.decode(&input[3..]);
    return text.into_owned();
  }

  if let Some(encoding) = text_heuristic::utf16_endian_from_sample(input) {
    let (text, _, _) = encoding.decode(input);
    return text.into_owned();
  }

  let (utf8, _, had_errors) = UTF_8.decode(input);
  if !had_errors {
    return utf8.into_owned();
  }

  let (cp1252, _, _) = WINDOWS_1252.decode(input);
  cp1252.into_owned()
}

pub fn extract_plain(input: &[u8]) -> Result<String, ExtractError> {
  Ok(decode_text_with_bom(input))
}

pub fn extract_plain_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  extract_plain(&read_path_bytes(path, max_bytes)?)
}

pub fn extract_csv(input: &[u8], delimiter: u8) -> Result<String, ExtractError> {
  extract_csv_reader(Cursor::new(input), delimiter)
}

pub fn extract_csv_from_path(path: &Path, delimiter: u8, max_bytes: usize) -> Result<String, ExtractError> {
  validate_path_size(path, max_bytes)?;
  let file = File::open(path).map_err(|err| ExtractError::Io(format!("csv open: {err}")))?;
  extract_csv_reader(BufReader::new(file), delimiter)
}

fn extract_csv_reader<R: Read>(reader: R, delimiter: u8) -> Result<String, ExtractError> {
  let mut reader = ReaderBuilder::new()
    .delimiter(delimiter)
    .has_headers(false)
    .from_reader(reader);

  let mut rows = Vec::new();
  for record in reader.records() {
    let record = record.map_err(|err| ExtractError::Parse(format!("csv: {err}")))?;
    let line = record.iter().map(str::trim).collect::<Vec<_>>().join(" | ");
    if !line.is_empty() {
      rows.push(line);
    }
  }

  Ok(rows.join("\n"))
}

pub fn extract_html(input: &[u8]) -> Result<String, ExtractError> {
  extract_html_reader(Cursor::new(input))
}

pub fn extract_html_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  validate_path_size(path, max_bytes)?;
  let file = File::open(path).map_err(|err| ExtractError::Io(format!("html open: {err}")))?;
  extract_html_reader(BufReader::new(file))
}

fn extract_html_reader<R: Read>(reader: R) -> Result<String, ExtractError> {
  let text = html2text::from_read(reader, 120).map_err(|err| ExtractError::Parse(format!("html: {err}")))?;
  Ok(text)
}

pub fn extract_xml(input: &[u8]) -> Result<String, ExtractError> {
  let mut reader = Reader::from_reader(Cursor::new(input));
  extract_xml_reader(&mut reader, Some(input))
}

pub fn extract_xml_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  validate_path_size(path, max_bytes)?;
  let file = File::open(path).map_err(|err| ExtractError::Io(format!("xml open: {err}")))?;
  let mut reader = Reader::from_reader(BufReader::new(file));
  extract_xml_reader(&mut reader, None)
}

fn extract_xml_reader(
  reader: &mut Reader<impl BufRead>,
  fallback_bytes: Option<&[u8]>,
) -> Result<String, ExtractError> {
  reader.config_mut().trim_text(true);

  let mut out = Vec::<String>::new();
  let mut buf = Vec::new();

  loop {
    match reader.read_event_into(&mut buf) {
      Ok(Event::Text(text)) => {
        let value: Cow<'_, str> = text
          .decode()
          .map_err(|err| ExtractError::Parse(format!("xml decode: {err}")))?;
        let trimmed = value.trim();
        if !trimmed.is_empty() {
          out.push(trimmed.to_string());
        }
      }
      Ok(Event::CData(cdata)) => {
        let value: Cow<'_, str> = cdata
          .decode()
          .map_err(|err| ExtractError::Parse(format!("xml cdata decode: {err}")))?;
        let trimmed = value.trim();
        if !trimmed.is_empty() {
          out.push(trimmed.to_string());
        }
      }
      Ok(Event::Eof) => break,
      Ok(_) => {}
      Err(err) => return Err(ExtractError::Parse(format!("xml: {err}"))),
    }
    buf.clear();
  }

  if out.is_empty() {
    if let Some(input) = fallback_bytes {
      return Ok(decode_text_with_bom(input));
    }
    return Err(ExtractError::EmptyResult);
  }

  Ok(out.join("\n"))
}
