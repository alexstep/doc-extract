use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use calamine::{open_workbook_auto, open_workbook_auto_from_rs, Reader};

use crate::error::ExtractError;
use crate::formats::open_file;
use crate::formats::zip_util::enforce_zip_uncompressed_budget;

pub fn extract(input: &[u8], max_bytes: usize) -> Result<String, ExtractError> {
  extract_reader(Cursor::new(input), max_bytes)
}

pub fn extract_from_path(path: &Path, max_bytes: usize) -> Result<String, ExtractError> {
  drop(guard_zip_budget(open_file(path)?, max_bytes)?);
  let mut workbook = open_workbook_auto(path)
    .map_err(|err| ExtractError::Parse(format!("spreadsheet open: {err}")))?;
  extract_workbook(&mut workbook)
}

pub fn extract_reader<R: Read + Seek + Clone>(
  reader: R,
  max_bytes: usize,
) -> Result<String, ExtractError> {
  let reader = guard_zip_budget(reader, max_bytes)?;
  let mut workbook = open_workbook_auto_from_rs(reader)
    .map_err(|err| ExtractError::Parse(format!("spreadsheet open: {err}")))?;
  extract_workbook(&mut workbook)
}

fn guard_zip_budget<R: Read + Seek>(mut reader: R, max_bytes: usize) -> Result<R, ExtractError> {
  let mut magic = [0_u8; 4];
  let read = reader
    .read(&mut magic)
    .map_err(|err| ExtractError::Io(format!("spreadsheet magic: {err}")))?;
  reader
    .seek(SeekFrom::Start(0))
    .map_err(|err| ExtractError::Io(format!("spreadsheet rewind: {err}")))?;
  if read >= 2 && magic[0] == b'P' && magic[1] == b'K' {
    return enforce_zip_uncompressed_budget(reader, max_bytes);
  }
  Ok(reader)
}

fn extract_workbook<RS: Read + Seek>(
  workbook: &mut calamine::Sheets<RS>,
) -> Result<String, ExtractError> {
  let mut out = Vec::new();

  for sheet_name in workbook.sheet_names().to_owned() {
    let range = workbook
      .worksheet_range(&sheet_name)
      .map_err(|err| ExtractError::Parse(format!("spreadsheet sheet {sheet_name}: {err}")))?;

    out.push(format!("Sheet: {sheet_name}"));
    for row in range.rows() {
      let line = row
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\t");
      if !line.trim().is_empty() {
        out.push(line);
      }
    }
  }

  Ok(out.join("\n"))
}
