use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyTypeError};
use pyo3::PyErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PandasMaxmindError {
    #[error(transparent)]
    MaxMindDBError(#[from] maxminddb::MaxMindDBError),

    #[error("invalid geo column name: {0}")]
    ParseColumnError(String),

    #[error("unsupported reader class")]
    UnsupportedReaderError,
}

impl From<PandasMaxmindError> for PyErr {
    fn from(e: PandasMaxmindError) -> Self {
        use PandasMaxmindError::*;

        match e {
            MaxMindDBError(_) => PyRuntimeError::new_err(e.to_string()),
            UnsupportedReaderError => PyTypeError::new_err(e.to_string()),
            ParseColumnError(_) => PyKeyError::new_err(e.to_string()),
        }
    }
}
