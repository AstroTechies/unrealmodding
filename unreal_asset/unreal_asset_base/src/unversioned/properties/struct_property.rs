//! Struct property

use std::mem::size_of;

use byteorder::WriteBytesExt;

use crate::reader::{ArchiveReader, ArchiveWriter};
use crate::unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter};
use crate::Error;

use super::{EPropertyType, UsmapPropertyDataTrait};

/// Struct property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapStructPropertyData {
    /// Struct type
    pub struct_type: String,
}

impl UsmapStructPropertyData {
    /// Read a `UsmapStructPropertyData` from an asset
    pub fn new<R: ArchiveReader>(asset: &mut UsmapReader<'_, '_, R>) -> Result<Self, Error> {
        let struct_type = asset.read_name()?;

        Ok(UsmapStructPropertyData { struct_type })
    }
}

impl UsmapPropertyDataTrait for UsmapStructPropertyData {
    fn write<W: ArchiveWriter>(&self, asset: &mut UsmapWriter<'_, '_, W>) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::StructProperty as u8)?;
        asset.write_name(&self.struct_type)?;
        Ok(size_of::<i32>() * 2)
    }

    fn get_property_type(&self) -> EPropertyType {
        EPropertyType::StructProperty
    }
}
