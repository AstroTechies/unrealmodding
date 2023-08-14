//! Enum property

use std::mem::size_of;

use byteorder::WriteBytesExt;

use crate::reader::{ArchiveReader, ArchiveWriter};
use crate::unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter};
use crate::Error;

use super::{EPropertyType, UsmapPropertyData, UsmapPropertyDataTrait};

/// Enum property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapEnumPropertyData {
    /// Inner property
    pub inner_property: Box<UsmapPropertyData>,
    /// Name
    pub name: String,
}

impl UsmapEnumPropertyData {
    /// Read a `UsmapEnumPropertyData` from an asset
    pub fn new<R: ArchiveReader>(asset: &mut UsmapReader<'_, '_, R>) -> Result<Self, Error> {
        let inner_property = UsmapPropertyData::new(asset)?;
        let name = asset.read_name()?;

        Ok(UsmapEnumPropertyData {
            inner_property: Box::new(inner_property),
            name,
        })
    }
}

impl UsmapPropertyDataTrait for UsmapEnumPropertyData {
    fn write<W: ArchiveWriter>(&self, asset: &mut UsmapWriter<'_, '_, W>) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::EnumProperty as u8)?;
        let size = self.inner_property.write(asset)?;
        asset.write_name(&self.name)?;

        Ok(size + size_of::<u8>() + size_of::<u32>() * 2)
    }

    fn get_property_type(&self) -> EPropertyType {
        EPropertyType::EnumProperty
    }
}
