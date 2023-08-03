//! Error type

#[cfg(feature = "read_write")]
use std::{
    io,
    string::{FromUtf16Error, FromUtf8Error},
};

#[cfg(feature = "read_write")]
use thiserror::Error;

/// Gets thrown when there is an error reading/writing an FString.
#[cfg(feature = "read_write")]
#[derive(Error, Debug)]
pub enum FStringError {
    /// String has invalid size
    #[error("Invalid string size {0} at position {1}")]
    InvalidStringSize(i32, u64),
    /// String has invalid terminator
    #[error("Invalid string terminator {0} at position {1}")]
    InvalidStringTerminator(u16, u64),
    /// String is not in the expected UTF-8 format
    #[error("Utf8 Error {0}")]
    Utf8(#[from] FromUtf8Error),
    /// String is not in the expected UTF-16 format
    #[error("Utf16 Error {0}")]
    Utf16(#[from] FromUtf16Error),
    /// Io Error
    #[error("Io Error {0}")]
    Io(#[from] io::Error),
}
