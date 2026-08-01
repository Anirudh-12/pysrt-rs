use thiserror::Error;

#[derive(Error, Debug)]
pub enum SrtError {
    #[error("Invalid time string: {0}")]
    InvalidTimeString(String),

    #[error("Invalid subtitle item: {0}")]
    InvalidItem(String),

    #[error("Invalid subtitle index: {0}")]
    InvalidIndex(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encoding error: {0}")]
    Encoding(String),
}

pub type Result<T> = std::result::Result<T, SrtError>;

#[cfg(feature = "python")]
impl From<SrtError> for pyo3::PyErr {
    fn from(err: SrtError) -> pyo3::PyErr {
        use pyo3::exceptions::PyValueError;
        // In python/mod.rs we also expose custom Python exception classes.
        PyValueError::new_err(err.to_string())
    }
}
