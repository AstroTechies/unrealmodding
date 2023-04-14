//! Array property

use std::mem::size_of;

use crate::{
    error::Error,
    unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter},
};

use super::{EPropertyType, UsmapPropertyData, UsmapPropertyDataTrait};

/// Array property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapArrayPropertyData {
    /// Inner array type
    pub inner_type: Box<UsmapPropertyData>,
}

impl UsmapArrayPropertyData {
    /// Read a `UsmapArrayPropertyData` from an asset
    pub fn new<Reader: UsmapReader>(asset: &mut Reader) -> Result<Self, Error> {
        let inner_type = UsmapPropertyData::new(asset)?;

        Ok(UsmapArrayPropertyData {
            inner_type: Box::new(inner_type),
        })
    }
}

impl UsmapPropertyDataTrait for UsmapArrayPropertyData {
    fn write<Writer: UsmapWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::ArrayProperty as u8)?;
        let size = self.inner_type.write(asset)?;
        Ok(size + size_of::<u8>())
    }
}
