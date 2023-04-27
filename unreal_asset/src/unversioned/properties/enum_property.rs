//! Enum property

use std::mem::size_of;

use crate::{
    error::Error,
    unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter},
};

use super::{EPropertyType, UsmapPropertyData, UsmapPropertyDataTrait};

/// Enum property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapEnumPropertyData {
    /// Inner property
    pub inner_property: Box<UsmapPropertyData>,
    /// Name
    pub name: Option<String>,
}

impl UsmapEnumPropertyData {
    /// Read a `UsmapEnumPropertyData` from an asset
    pub fn new<Reader: UsmapReader>(asset: &mut Reader) -> Result<Self, Error> {
        let inner_property = UsmapPropertyData::new(asset)?;
        let name = asset.read_name()?;

        Ok(UsmapEnumPropertyData {
            inner_property: Box::new(inner_property),
            name,
        })
    }
}

impl UsmapPropertyDataTrait for UsmapEnumPropertyData {
    fn write<Writer: UsmapWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::EnumProperty as u8)?;
        let size = self.inner_property.write(asset)?;
        asset.write_name(self.name.as_deref())?;

        Ok(size + size_of::<u8>() + size_of::<u32>() * 2)
    }

    fn get_property_type(&self) -> EPropertyType {
        EPropertyType::EnumProperty
    }
}
