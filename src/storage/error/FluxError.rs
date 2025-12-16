use std::{fmt, io};

pub type Result<T> = std::result::Result<T, FluxError>;

#[derive(Debug)]
pub enum FluxError{
    Io(std::io::Error),

    /// The DB file / page / record bytes are not valid for the expected format.
    CorruptData(&'static str),

    /// A numeric tag/enum value is not recognized by this version.
    InvalidEnumValue{
        what: &'static str,
        value: u64,
    },

    /// A string field in storage is not valid UTF-8.
    InvalidUtf8(&'static str),

    /// Something expected to exist (slot/record/etc.) is missing.
    NotFound(&'static str),
}

impl fmt::Display for FluxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FluxError::Io(e) => write!(f, "I/O error: {}", e),
            FluxError::CorruptData(msg) => write!(f, "corrupt/invalid data: {}", msg),
            FluxError::InvalidEnumValue { what, value } => {
                write!(f, "invalid {} value: {}", what, value)
            }
            FluxError::InvalidUtf8(what) => write!(f, "invalid UTF-8 in {}", what),
            FluxError::NotFound(what) => write!(f, "not found: {}", what),
        }
    }
}

impl std::error::Error for FluxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FluxError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for FluxError {
    fn from(value: io::Error) -> Self {
        FluxError::Io(value)
    }
}