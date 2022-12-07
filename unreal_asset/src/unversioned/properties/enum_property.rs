use std::mem::size_of;

use crate::{
    error::Error,
    unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter},
};

use super::{EPropertyType, UsmapPropertyData, UsmapPropertyDataTrait};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapEnumPropertyData {
    pub inner_property: Box<UsmapPropertyData>,
    pub name: String,
}

impl UsmapEnumPropertyData {
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
        asset.write_name(&self.name)?;

        Ok(size + size_of::<u8>() + size_of::<u32>() * 2)
    }
}
