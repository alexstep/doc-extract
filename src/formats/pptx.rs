use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;

use zip::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::ExtractError;
use crate::formats::open_file;
use crate::formats::zip_util::read_zip_entry_limited;

const MAX_TOTAL_SLIDES_SIZE: usize = 10 * 1024 * 1024;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  extract_reader(Cursor::new(input))
}

pub fn extract_from_path(path: &Path) -> Result<String, ExtractError> {
  extract_reader(open_file(path)?)
}

pub fn extract_reader<R: Read + Seek>(reader: R) -> Result<String, ExtractError> {
  let mut archive =
    ZipArchive::new(reader).map_err(|err| ExtractError::Parse(format!("pptx zip: {err}")))?;

  let mut slides = Vec::new();
  let mut total_size: usize = 0;

  for index in 0..archive.len() {
    let mut entry = archive
      .by_index(index)
      .map_err(|err| ExtractError::Parse(format!("pptx entry {index}: {err}")))?;
    let name = entry.name().to_string();
    if !name.starts_with("ppt/slides/slide") || !name.ends_with(".xml") {
      continue;
    }

    let slide_number = name
      .trim_start_matches("ppt/slides/slide")
      .trim_end_matches(".xml")
      .parse::<usize>()
      .unwrap_or(slides.len() + 1);

    let xml = read_zip_entry_limited(&mut entry, &format!("pptx read {name}"))?;

    total_size = total_size.saturating_add(xml.len());
    if total_size > MAX_TOTAL_SLIDES_SIZE {
      break;
    }

    let text = super::text::extract_xml(&xml)?;
    if !text.trim().is_empty() {
      slides.push((slide_number, text));
    }
  }

  slides.sort_by_key(|(number, _)| *number);

  if slides.is_empty() {
    return Err(ExtractError::EmptyResult);
  }

  Ok(slides
    .into_iter()
    .map(|(number, text)| format!("Slide {number}:\n{text}"))
    .collect::<Vec<_>>()
    .join("\n\n"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_slide_text() {
    let slide = br#"<?xml version="1.0"?><p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree><a:p><a:r><a:t>Launch Party</a:t></a:r></a:p></p:spTree></p:cSld></p:sld>"#;
    let mut buffer = Cursor::new(Vec::new());
    {
      let mut zip = ZipWriter::new(&mut buffer);
      let options = SimpleFileOptions::default();
      zip.start_file("ppt/slides/slide1.xml", options).unwrap();
      zip.write_all(slide).unwrap();
      zip.finish().unwrap();
    }

    let text = extract(buffer.into_inner().as_slice()).unwrap();
    assert!(text.contains("Launch Party"));
  }
}
