use std::io::{Cursor, Read, Seek};
use std::path::Path;

use calamine::{open_workbook_auto, open_workbook_auto_from_rs, Reader};

use crate::error::ExtractError;

pub fn extract(input: &[u8]) -> Result<String, ExtractError> {
  extract_reader(Cursor::new(input))
}

pub fn extract_from_path(path: &Path) -> Result<String, ExtractError> {
  let mut workbook =
    open_workbook_auto(path).map_err(|err| ExtractError::Parse(format!("spreadsheet open: {err}")))?;
  extract_workbook(&mut workbook)
}

pub fn extract_reader<R: Read + Seek + Clone>(reader: R) -> Result<String, ExtractError> {
  let mut workbook =
    open_workbook_auto_from_rs(reader).map_err(|err| ExtractError::Parse(format!("spreadsheet open: {err}")))?;
  extract_workbook(&mut workbook)
}

fn extract_workbook<RS: Read + Seek>(workbook: &mut calamine::Sheets<RS>) -> Result<String, ExtractError> {
  let mut out = Vec::new();

  for sheet_name in workbook.sheet_names().to_owned() {
    let range = workbook
      .worksheet_range(&sheet_name)
      .map_err(|err| ExtractError::Parse(format!("spreadsheet sheet {sheet_name}: {err}")))?;

    out.push(format!("Sheet: {sheet_name}"));
    for row in range.rows() {
      let line = row.iter().map(ToString::to_string).collect::<Vec<_>>().join("\t");
      if !line.trim().is_empty() {
        out.push(line);
      }
    }
  }

  Ok(out.join("\n"))
}
