//! Unreal types

pub mod fname;
use byteorder::{ReadBytesExt, WriteBytesExt};
pub use fname::FName;

pub mod movie;
pub mod vector;

use std::hash::Hash;

use crate::reader::{ArchiveReader, ArchiveWriter};
use crate::Error;
use crate::Guid;

/// Serialized name header
/// Used when reading name batches in >=UE5
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SerializedNameHeader {
    /// Is wide
    pub is_wide: bool,
    /// Name header length
    pub len: i32,
}

impl SerializedNameHeader {
    /// Read a `SerializedNameHeader` from an archive
    pub fn read<Reader: ArchiveReader<impl PackageIndexTrait> + ?Sized>(
        reader: &mut Reader,
    ) -> Result<SerializedNameHeader, Error> {
        let (first_byte, second_byte) = (reader.read_u8()?, reader.read_u8()?);

        Ok(SerializedNameHeader {
            is_wide: (first_byte & 0x80) > 0,
            len: (((first_byte & 0x7f) as i32) << 8) + second_byte as i32,
        })
    }

    /// Write a `SerializedNameHeader` to an archive
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        writer: &mut Writer,
    ) -> Result<(), Error> {
        let is_wide = match self.is_wide {
            true => 1u8,
            false => 0u8,
        };
        let first_byte = is_wide << 7 | (self.len >> 8) as u8;
        let second_byte = self.len as u8;

        writer.write_u8(first_byte)?;
        writer.write_u8(second_byte)?;

        Ok(())
    }
}

/// PackageIndexTrait is used to group PackageIndex and PackageObjectIndex together
/// This is useful for exports to share code between UAsset/IoStore implementations
pub trait PackageIndexTrait: std::fmt::Debug + Copy + Clone + PartialEq + Eq + ToString {
    /// Check if this index is an import
    fn is_import(&self) -> bool;
    /// Check if this index is an export
    fn is_export(&self) -> bool;
}

/// PackageIndex is one of the most important structs in UE4
///
/// It is basically a reference into an import/export table
/// which helps "glue" together assets.
///
/// If a PackageIndex is negative it's an index inside an import table
/// if it's positive it's an index inside an export table.
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

impl PackageIndexTrait for PackageIndex {
    fn is_import(&self) -> bool {
        self.index < 0
    }

    fn is_export(&self) -> bool {
        self.index > 0
    }
}

impl ToString for PackageIndex {
    fn to_string(&self) -> String {
        self.index.to_string()
    }
}

/// Create a Guid from 4 u32 values
// #[rustfmt::skip]
// pub const fn new_guid(a: u32, b: u32, c: u32, d: u32) -> Guid {
//     Guid
// }

/// Create a default Guid filled with all zeros
// pub fn default_guid() -> Guid {
//     new_guid(0, 0, 0, 0)
// }

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
