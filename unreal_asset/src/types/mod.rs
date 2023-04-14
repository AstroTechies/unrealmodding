//! Unreal types

pub mod movie;
pub mod vector;

use crate::error::Error;

/// FName is used to store most of the Strings in UE4.
///
/// They are represented by an index+instance number inside a string table inside the asset file.
///
/// Here we are representing them by a string and an instance number.
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct FName {
    /// FName content
    pub content: String,
    /// FName index
    pub index: i32,
}

/// Convert implementer to `FName`
pub trait ToFName {
    /// Convert to `FName`
    fn to_fname(&self) -> FName;
}

impl FName {
    /// Create a new `FName` instance with an index
    pub fn new(content: String, index: i32) -> Self {
        FName { content, index }
    }

    /// Create a new `FName` instance from a slice with an index of 0
    pub fn from_slice(content: &str) -> Self {
        FName {
            content: content.to_string(),
            index: 0,
        }
    }
}

/// PackageIndex is one of the most important structs in UE4
///
/// It is basically a reference into an import/export table
/// which helps "glue" together assets.
///
/// If a PackageIndex is negative it's an index inside an export table
/// if it's positive it's an index inside an import table.
///
/// When PackageIndex is 0 it makes for a non-existent link.
#[derive(Debug, Hash, Copy, Clone, Default, PartialEq, Eq)]
pub struct PackageIndex {
    /// Index
    pub index: i32,
}

impl PackageIndex {
    /// Create a new `PackageIndex`
    pub fn new(index: i32) -> Self {
        PackageIndex { index }
    }

    /// Check if this index is an import
    pub fn is_import(&self) -> bool {
        self.index < 0
    }

    /// Check if this index is an export
    pub fn is_export(&self) -> bool {
        self.index > 0
    }

    /// Create a `PackageIndex` from an import index
    pub fn from_import(import_index: i32) -> Result<Self, Error> {
        match import_index < 0 {
            true => Err(Error::invalid_package_index(
                "Import index must be bigger than zero".to_string(),
            )),
            false => Ok(PackageIndex::new(-import_index - 1)),
        }
    }

    /// Create a `PackageIndex` from an export index
    pub fn from_export(export_index: i32) -> Result<Self, Error> {
        match export_index < 0 {
            true => Err(Error::invalid_package_index(
                "Export index must be greater than zero".to_string(),
            )),
            false => Ok(PackageIndex::new(export_index + 1)),
        }
    }
}

/// Guid
pub type Guid = [u8; 16];

/// Create a Guid from 4 u32 values
#[rustfmt::skip]
pub const fn new_guid(a: u32, b: u32, c: u32, d: u32) -> Guid {
    [
        (a & 0xff) as u8, ((a >> 8) & 0xff) as u8, ((a >> 16) & 0xff) as u8, ((a >> 24) & 0xff) as u8,
        (b & 0xff) as u8, ((b >> 8) & 0xff) as u8, ((b >> 16) & 0xff) as u8, ((b >> 24) & 0xff) as u8,
        (c & 0xff) as u8, ((c >> 8) & 0xff) as u8, ((c >> 16) & 0xff) as u8, ((c >> 24) & 0xff) as u8,
        (d & 0xff) as u8, ((d >> 8) & 0xff) as u8, ((d >> 16) & 0xff) as u8, ((d >> 24) & 0xff) as u8
    ]
}

/// Create a default Guid filled with all zeroes
pub fn default_guid() -> Guid {
    new_guid(0, 0, 0, 0)
}

/// Asset generation info
#[derive(Debug)]
pub struct GenerationInfo {
    /// Export count
    pub export_count: i32,
    /// Name count
    pub name_count: i32,
}

impl GenerationInfo {
    /// Create a new `GenerationInfo` instance
    pub fn new(export_count: i32, name_count: i32) -> Self {
        GenerationInfo {
            export_count,
            name_count,
        }
    }
}
