use pyo3::exceptions::PyKeyError;
use pyo3::PyErr;
use thiserror::Error;

// todo: into PyErr
#[derive(Error, Debug)]
pub enum PandasMaxmindError {
  #[error("invalid geo column name: {0}")]
  ParseColumnError(String)
}

impl From<PandasMaxmindError> for PyErr {
  fn from(e: PandasMaxmindError) -> Self {
    match e {
      PandasMaxmindError::ParseColumnError(_) => PyKeyError::new_err(e.to_string())
    }
  }
}
