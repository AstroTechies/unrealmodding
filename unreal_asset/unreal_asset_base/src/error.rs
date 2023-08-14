//! All errors thrown by unreal_asset

use std::io;
use std::string::{FromUtf16Error, FromUtf8Error};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;
use unreal_helpers::error::FStringError;

use crate::custom_version::FAssetRegistryVersionType;
use crate::reader::ArchiveType;
use crate::unversioned::Ancestry;

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
    /// Name map index out of range
    #[error("Name map index out of range, name map size: {0}, got: {1}")]
    NameMapIndexOutOfRange(usize, i32),
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

    /// Create an `UsmapError` for a case where the name map index was out of range
    pub fn name_map_index_out_of_range(name_map_size: usize, index: i32) -> Self {
        UsmapError::NameMapIndexOutOfRange(name_map_size, index)
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
    /// An unversioned property's type could not be determined
    #[error("Cannot determine property type for property {0} ancestry {1}")]
    NoType(Box<str>, Box<str>),
    /// An unversioned property was found without loaded mappings
    #[error("Cannot deseralize an unversioned property without loaded mappings")]
    NoMappings,
    /// A usmap mapping for an unversioned property was not found
    #[error("No mapping for unversioned property {0} ancestry {1}")]
    NoMapping(Box<str>, Box<str>),
    /// Tried to read an unversioned property with no parent_name specified
    #[error("Tried to read an unversioned property with parent_name: None")]
    NoParent,
    /// An unversioned property was found, but no unversioned header was provided
    #[error("Tried to read an unversioned property without an unversioned header")]
    NoUnversionedHeader,
    /// An unversioned property schema was not found
    #[error("Unversioned property schema for {0} at index {1} was not found")]
    NoSchema(Box<str>, usize),
    /// Other
    #[error("{0}")]
    Other(Box<str>),
}

impl PropertyError {
    /// Create a `PropertyError` for a property where a header was expected when serializing, but none was found
    pub fn headerless() -> Self {
        PropertyError::HeaderlessProperty
    }

    /// Create a `PropertyError` for an unversioned property for which type could not be determined
    pub fn no_type(name: &str, ancestry: &Ancestry) -> Self {
        PropertyError::NoType(
            name.to_string().into_boxed_str(),
            ancestry
                .ancestry
                .iter()
                .map(|e| e.get_owned_content())
                .collect::<Vec<_>>()
                .join("/")
                .into_boxed_str(),
        )
    }

    /// Create a `PropertyError` for an unversioned property that failed to deserialize because no mappings were loaded
    pub fn no_mappings() -> Self {
        PropertyError::NoMappings
    }

    /// Create a `PropertyError` for an unversioned property that did not have a mapping for a certain ancestry
    pub fn no_mapping(name: &str, ancestry: &Ancestry) -> Self {
        PropertyError::NoMapping(
            name.to_string().into_boxed_str(),
            ancestry
                .ancestry
                .iter()
                .map(|e| e.get_owned_content())
                .collect::<Vec<_>>()
                .join("/")
                .into_boxed_str(),
        )
    }

    /// Create a `PropertyError` for an unversioned property with no parent name specified
    pub fn no_parent() -> Self {
        PropertyError::NoParent
    }

    /// Create a `PropertyError` for an unversioned property with no unversioned header specified
    pub fn no_unversioned_header() -> Self {
        PropertyError::NoUnversionedHeader
    }

    /// Create a `PropertyError` for an unversioned property for which a schema entry was not found
    pub fn no_schema(name: String, index: usize) -> Self {
        PropertyError::NoSchema(name.into_boxed_str(), index)
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

/// Thrown when an FName error occured
#[derive(Error, Debug)]
pub enum FNameError {
    /// An FName pointer was out of range
    #[error("Cannot read FName, index: {0}, name map size: {1}")]
    OutOfRange(i32, usize),
    /// Tried to serialize a "dummy" FName
    #[error("Cannot serialize a dummy FName, content: {0}, number: {1}")]
    DummySerialize(Box<str>, i32),
}

impl FNameError {
    /// Create an `FNameError` when an FName pointer was out of range
    pub fn out_of_range(index: i32, name_map_size: usize) -> Self {
        FNameError::OutOfRange(index, name_map_size)
    }

    /// Create an `FNameError` when a "dummy" FName tried to serialize
    pub fn dummy_serialize(content: &str, number: i32) -> Self {
        Self::DummySerialize(content.to_string().into_boxed_str(), number)
    }
}

/// Zen-specific error type
#[derive(Error, Debug)]
pub enum ZenError {
    /// No mappings were provided before serialization
    #[error("No mappings were provided before serialization")]
    NoMappings,
    /// Object version was not set before serialization
    #[error("No engine version was set before serialization")]
    NoObjectVersion,
}

/// IoStore error
#[derive(Error, Debug)]
pub enum IoStoreError {
    /// Invalid toc magic
    #[error("Invalid .utoc magic, got: {0:?}")]
    InvalidTocMagic([u8; 16]),
    /// Invalid header size
    #[error("Invalid .utoc header size, expected: {0}, got: {1}")]
    InvalidTocHeaderSize(u32, u32),
    /// Invalid enum value
    #[error("{0}")]
    InvalidEnumValue(Box<str>),
    /// Tried to get a non-existent file from an `IoStoreFileProvider`
    #[error("Tried to get a non-existent file {0}")]
    NoFile(Box<str>),
    /// Tried to get a non-existent IoStore chunk
    #[error("Chunk with name {0} doesn't exist")]
    NoChunk(Box<str>),

    /// No encryption key was provided for an encrypted file
    #[error("No encryption key was provided for an encrypted file")]
    NoEncryptionKey,

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
}

impl IoStoreError {
    /// Create a new `InvalidTocHeaderSize` error
    pub fn invalid_toc_header_size(expected: u32, got: u32) -> Self {
        IoStoreError::InvalidTocHeaderSize(expected, got)
    }

    /// Create a new `NoChunk` error
    pub fn no_chunk(name: &str) -> Self {
        IoStoreError::NoChunk(name.to_string().into_boxed_str())
    }
}

impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for IoStoreError {
    fn from(e: TryFromPrimitiveError<T>) -> Self {
        IoStoreError::InvalidEnumValue(e.to_string().into_boxed_str())
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
    /// An `FNameError` occured
    #[error(transparent)]
    FName(#[from] FNameError),
    /// Archive type didn't match the expected type
    #[error("Archive type mismatch, expected {0}, got {1}")]
    ArchiveTypeMismatch(Box<str>, ArchiveType),
    /// Cityhash64 hash collision
    #[error("Cityhash64 name collision for hash {0}, string {1}")]
    Cityhash64Collision(u64, Box<str>),
    /// Hash mismatch when reading a name batch
    #[error("Hash mismatch when reading a name batch, expected hash {0}, got {1}, string {2}")]
    NameBatchHashMismatch(u64, u64, Box<str>),
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
    /// A `IoStoreError` occured
    #[error(transparent)]
    IoStore(#[from] IoStoreError),

    /// Tried to decompress data with an unknown compression method
    #[error("Unknown compression method {0}")]
    UnknownCompressionMethod(Box<str>),
    /// An LZ4 decompression error occured
    #[error(transparent)]
    Lz4(#[from] lz4_flex::block::DecompressError),
    /// Oodle decompression failed
    #[error("Oodle decompression failed")]
    Oodle,
    /// Oodle library not initialized
    #[error("Oodle decompression library is not initialized")]
    OodleNotInitialized,

    /// A `ZenError` occured
    #[error(transparent)]
    Zen(#[from] ZenError),
}

impl Error {
    /// Create an `Error` for a case where expected data was not found
    pub fn no_data(msg: String) -> Self {
        Error::NoData(msg.into_boxed_str())
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

    /// Create an `Error` for a Cityhash64 hash collision
    pub fn cityhash64_collision(hash: u64, value: String) -> Self {
        Error::Cityhash64Collision(hash, value.into_boxed_str())
    }

    /// Create an `Error` for a hash mismatch when reading a name batch
    pub fn name_batch_hash_mismatch(expected: u64, got: u64, value: String) -> Self {
        Error::NameBatchHashMismatch(expected, got, value.into_boxed_str())
    }

    /// Create an `Error` for an archive type mismatch
    pub fn archive_type_mismatch(expected: &[ArchiveType], got: ArchiveType) -> Self {
        let expected = expected
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("/");
        Error::ArchiveTypeMismatch(expected.into_boxed_str(), got)
    }
}

impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for Error {
    fn from(e: TryFromPrimitiveError<T>) -> Self {
        Error::InvalidEnumValue(e.to_string().into_boxed_str())
    }
}
