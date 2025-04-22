// src/utils/error.rs
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)] // Ensure Clone is derived
pub struct BinaryDataException(String);

impl BinaryDataException {
    pub fn new(message: String) -> Self {
        Self(message)
    }

    pub fn from_str(message: &str) -> Self {
        Self(message.to_string())
    }
}

impl fmt::Display for BinaryDataException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for BinaryDataException {
    fn from(s: String) -> Self {
        BinaryDataException::new(s)
    }
}
impl Error for BinaryDataException {}

pub type Result<T> = std::result::Result<T, BinaryDataException>;