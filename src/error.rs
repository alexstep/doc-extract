use napi::{Error, Status};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractError {
  #[error("Input exceeds max size")]
  InputTooLarge,
  #[error("Unsupported format: {0}")]
  UnsupportedFormat(String),
  #[error("Parser failed: {0}")]
  Parse(String),
  #[error("Extracted text is empty")]
  EmptyResult,
  #[error("Background task failed")]
  TaskJoin,
  #[error("{0}")]
  Io(String),
}

impl From<ExtractError> for Error {
  fn from(err: ExtractError) -> Self {
    let status = match &err {
      ExtractError::InputTooLarge | ExtractError::UnsupportedFormat(_) => Status::InvalidArg,
      ExtractError::Parse(_) | ExtractError::EmptyResult | ExtractError::Io(_) => Status::GenericFailure,
      ExtractError::TaskJoin => Status::GenericFailure,
    };
    Error::new(status, err.to_string())
  }
}
