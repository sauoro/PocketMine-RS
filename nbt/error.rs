// src/nbt/error.rs
#![allow(dead_code)]

use std::error::Error;
use std::fmt;
use crate::utils; // Adjusted path

#[derive(Debug)]
pub enum NbtError {
    IoError(utils::error::BinaryDataException), // Use qualified path
    DataError(String),
    InvalidTagValue(String),
    UnexpectedTagType(String),
    NoSuchTag(String),
    DepthLimitExceeded(String),
    InvalidOperation(String),
    Utf8Error(std::string::FromUtf8Error),
    TryFromIntError(std::num::TryFromIntError),
}

impl NbtError {
    pub fn new_data_error(message: &str) -> Self {
        NbtError::DataError(message.to_string())
    }
    pub fn new_invalid_tag_value(message: &str) -> Self {
        NbtError::InvalidTagValue(message.to_string())
    }
    pub fn new_unexpected_tag_type(message: &str) -> Self {
        NbtError::UnexpectedTagType(message.to_string())
    }
    pub fn new_no_such_tag(message: &str) -> Self {
        NbtError::NoSuchTag(message.to_string())
    }
    pub fn new_depth_limit_exceeded(message: &str) -> Self {
        NbtError::DepthLimitExceeded(message.to_string())
    }
    pub fn new_invalid_operation(message: &str) -> Self {
        NbtError::InvalidOperation(message.to_string())
    }
}

impl fmt::Display for NbtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NbtError::IoError(e) => write!(f, "NBT IO Error: {}", e),
            NbtError::DataError(msg) => write!(f, "NBT Data Error: {}", msg),
            NbtError::InvalidTagValue(msg) => write!(f, "NBT Invalid Tag Value: {}", msg),
            NbtError::UnexpectedTagType(msg) => write!(f, "NBT Unexpected Tag Type: {}", msg),
            NbtError::NoSuchTag(msg) => write!(f, "NBT No Such Tag: {}", msg),
            NbtError::DepthLimitExceeded(msg) => write!(f, "NBT Depth Limit Exceeded: {}", msg),
            NbtError::InvalidOperation(msg) => write!(f, "NBT Invalid Operation: {}", msg),
            NbtError::Utf8Error(e) => write!(f, "NBT UTF-8 Error: {}", e),
            NbtError::TryFromIntError(e) => write!(f, "NBT Integer Conversion Error: {}", e),
        }
    }
}

impl Error for NbtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NbtError::IoError(e) => Some(e),
            NbtError::Utf8Error(e) => Some(e),
            NbtError::TryFromIntError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<utils::error::BinaryDataException> for NbtError {
    fn from(err: utils::error::BinaryDataException) -> Self {
        NbtError::IoError(err)
    }
}

impl From<std::string::FromUtf8Error> for NbtError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        NbtError::Utf8Error(err)
    }
}

impl From<std::num::TryFromIntError> for NbtError {
    fn from(err: std::num::TryFromIntError) -> Self {
        NbtError::TryFromIntError(err)
    }
}

pub type Result<T> = std::result::Result<T, NbtError>;