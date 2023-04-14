//! All errors thrown by unreal_asset

use std::io;
use std::string::{FromUtf16Error, FromUtf8Error};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;
use unreal_helpers::error::FStringError;

use crate::custom_version::FAssetRegistryVersionType;

/// Thrown when kismet bytecode failed to deserialize
#[derive(Error, Debug)]
pub enum KismetError {
    /// Kismet token was invalid
    #[error("{0}")]
    InvalidToken(Box<str>),
    /// Unknown kismet expression
    #[error("{0}")]
    UnknownExpression(Box<str>),
}

impl KismetError {
    /// Create a `KismetError` for an invalid token
    pub fn token(msg: String) -> Self {
        KismetError::InvalidToken(msg.into_boxed_str()) // todo: maybe not allocate a string
    }

    /// Create a `KismetError` for an unknown expression
    pub fn expression(msg: String) -> Self {
        KismetError::UnknownExpression(msg.into_boxed_str())
    }
}

/// Thrown when a usmap file failed to deserialize
#[derive(Error, Debug)]
pub enum UsmapError {
    /// Unsupported usmap compression
    #[error("Unsupported compression: {0}")]
    UnsupportedCompression(u8),
    /// Invalid compressiondata
    #[error("Invalid compression data")]
    InvalidCompressionData,
}

impl UsmapError {
    /// Create an `UsmapError` for an unsupported compresion
    pub fn unsupported_compression(compression: u8) -> Self {
        UsmapError::UnsupportedCompression(compression)
    }

    /// Create an `UsmapError` for invalid compression data
    pub fn invalid_compression_data() -> Self {
        UsmapError::InvalidCompressionData
    }
}

/// Thrown when asset registry failed to deserialize
#[derive(Error, Debug)]
pub enum RegistryError {
    /// Invalid registry index
    #[error("Invalid index: {0}")]
    InvalidIndex(i32),
    /// Invalid registry value for a given version
    #[error("Invalid value {0} for asset registry with version {1}")]
    Version(Box<str>, FAssetRegistryVersionType),
    /// Other
    #[error("{0}")]
    Other(Box<str>),
}

impl RegistryError {
    /// Create a `RegistryError` for an invalid registry index
    pub fn index(index: i32) -> Self {
        RegistryError::InvalidIndex(index)
    }

    /// Create a `RegistryError` for an invalid value for a given version
    pub fn version(msg: String, version: FAssetRegistryVersionType) -> Self {
        RegistryError::Version(msg.into_boxed_str(), version)
    }

    /// Create an other `RegistryError`
    pub fn other(msg: String) -> Self {
        RegistryError::Other(msg.into_boxed_str())
    }
}

/// Thrown when a property failed to deserialize
#[derive(Error, Debug)]
pub enum PropertyError {
    /// Header data was expected when serializing, but none was found
    #[error("include_header: true on headerless property")]
    HeaderlessProperty,
    /// A field was None when Some(...) was expected
    #[error("{0} is None, expected {1}")]
    PropertyFieldNone(Box<str>, Box<str>),
    /// A `StructProperty` is invalid
    #[error("{0}")]
    InvalidStruct(Box<str>),
    /// An `ArrayProperty` is invalid
    #[error("{0}")]
    InvalidArrayType(Box<str>),
    /// Other
    #[error("{0}")]
    Other(Box<str>),
}

impl PropertyError {
    /// Create a `PropertyError` for a property where a header was expected when serializing, but none was found
    pub fn headerless() -> Self {
        PropertyError::HeaderlessProperty
    }

    /// Create a `PropertyError` for a field that was expected to have a value, but was None
    pub fn property_field_none(field_name: &str, expected: &str) -> Self {
        PropertyError::PropertyFieldNone(
            field_name.to_string().into_boxed_str(),
            expected.to_string().into_boxed_str(),
        )
    }

    /// Create a `PropertyError` for an invalid `StructProperty`
    pub fn invalid_struct(msg: String) -> Self {
        PropertyError::InvalidStruct(msg.into_boxed_str())
    }

    /// Create a `PropertyError` for an invalid `ArrayProperty`
    pub fn invalid_array(msg: String) -> Self {
        PropertyError::InvalidArrayType(msg.into_boxed_str())
    }

    /// Create an other `PropertyError`
    pub fn other(msg: String) -> Self {
        PropertyError::Other(msg.into_boxed_str())
    }
}

/// Error type
#[derive(Error, Debug)]
pub enum Error {
    /// An `std::io::Error` occured
    #[error(transparent)]
    Io(#[from] io::Error),
    /// An `FStringError` occured
    #[error(transparent)]
    FString(#[from] FStringError),
    /// A `FromUtf8Error` occured
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
    /// A `FromUtf16Error` occured
    #[error(transparent)]
    Utf16(#[from] FromUtf16Error),
    /// Expected data was not found
    #[error("{0}")]
    NoData(Box<str>),
    /// An FName pointer was out of range
    #[error("Cannot read FName, index: {0}, name map size: {1}")]
    FName(i32, usize),
    /// The file is invalid
    #[error("{0}")]
    InvalidFile(Box<str>),
    /// A package index is invalid
    #[error("{0}")]
    InvalidPackageIndex(Box<str>),
    /// An enum value is invalid
    #[error("{0}")]
    InvalidEnumValue(Box<str>),
    /// Part of the library is not implemented
    #[error("{0}")]
    Unimplemented(Box<str>),
    /// A `KismetError` occured
    #[error(transparent)]
    Kismet(#[from] KismetError),
    /// A `PropertyError` occcured
    #[error(transparent)]
    Property(#[from] PropertyError),
    /// A `RegistryError` occured
    #[error(transparent)]
    Registry(#[from] RegistryError),
    /// A `UsmapError` occured
    #[error(transparent)]
    Usmap(#[from] UsmapError),
}

impl Error {
    /// Create an `Error` for a case where expected data was not found
    pub fn no_data(msg: String) -> Self {
        Error::NoData(msg.into_boxed_str())
    }

    /// Create an `Error` when an FName pointer was out of range
    pub fn fname(index: i32, name_map_size: usize) -> Self {
        Error::FName(index, name_map_size)
    }

    /// Create an `Error` when the file was invalid
    pub fn invalid_file(msg: String) -> Self {
        Error::InvalidFile(msg.into_boxed_str())
    }

    /// Create an `Error` when a package index is invalid
    pub fn invalid_package_index(msg: String) -> Self {
        Error::InvalidPackageIndex(msg.into_boxed_str())
    }

    /// Create an `Error` when a part of the library is not implemented
    pub fn unimplemented(msg: String) -> Self {
        Error::Unimplemented(msg.into_boxed_str())
    }
}

impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for Error {
    fn from(e: TryFromPrimitiveError<T>) -> Self {
        Error::InvalidEnumValue(e.to_string().into_boxed_str())
    }
}
