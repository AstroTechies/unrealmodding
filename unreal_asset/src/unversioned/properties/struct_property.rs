use std::mem::size_of;

use crate::{
    error::Error,
    unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter},
};

use super::{EPropertyType, UsmapPropertyDataTrait};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapStructPropertyData {
    pub struct_type: String,
}

impl UsmapStructPropertyData {
    pub fn new<Reader: UsmapReader>(asset: &mut Reader) -> Result<Self, Error> {
        let struct_type = asset.read_name()?;

        Ok(UsmapStructPropertyData { struct_type })
    }
}

impl UsmapPropertyDataTrait for UsmapStructPropertyData {
    fn write<Writer: UsmapWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::StructProperty as u8)?;
        asset.write_name(&self.struct_type)?;
        Ok(size_of::<i32>() * 2)
    }
}
