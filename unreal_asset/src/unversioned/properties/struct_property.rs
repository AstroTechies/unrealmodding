//! Struct property

use std::mem::size_of;

use crate::{
    error::Error,
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter},
};

use super::{EPropertyType, UsmapPropertyDataTrait};

/// Struct property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapStructPropertyData {
    /// Struct type
    pub struct_type: String,
}

impl UsmapStructPropertyData {
    /// Read a `UsmapStructPropertyData` from an asset
    pub fn new<'parent_reader, 'asset, R: ArchiveReader>(
        asset: &mut UsmapReader<'parent_reader, 'asset, R>,
    ) -> Result<Self, Error> {
        let struct_type = asset.read_name()?;

        Ok(UsmapStructPropertyData { struct_type })
    }
}

impl UsmapPropertyDataTrait for UsmapStructPropertyData {
    fn write<'parent_writer, 'asset, W: ArchiveWriter>(
        &self,
        asset: &mut UsmapWriter<'parent_writer, 'asset, W>,
    ) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::StructProperty as u8)?;
        asset.write_name(&self.struct_type)?;
        Ok(size_of::<i32>() * 2)
    }

    fn get_property_type(&self) -> EPropertyType {
        EPropertyType::StructProperty
    }
}
