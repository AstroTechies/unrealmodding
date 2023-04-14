//! Set property

use crate::{
    error::Error,
    unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter},
};

use super::{EPropertyType, UsmapPropertyData, UsmapPropertyDataTrait};

/// Set property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapSetPropertyData {
    /// Inner type
    pub inner_type: Box<UsmapPropertyData>,
}

impl UsmapSetPropertyData {
    /// Read a `UsmapSetPropertyData` from an asset
    pub fn new<Reader: UsmapReader>(asset: &mut Reader) -> Result<Self, Error> {
        let inner_type = UsmapPropertyData::new(asset)?;

        Ok(UsmapSetPropertyData {
            inner_type: Box::new(inner_type),
        })
    }
}

impl UsmapPropertyDataTrait for UsmapSetPropertyData {
    fn write<Writer: UsmapWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::SetProperty as u8)?;
        todo!()
    }
}
