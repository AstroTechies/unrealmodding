//! Map property

use std::mem::size_of;

use crate::{
    error::Error,
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    unversioned::{usmap_reader::UsmapReader, usmap_writer::UsmapWriter},
};

use super::{EPropertyType, UsmapPropertyData, UsmapPropertyDataTrait};

/// Map property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapMapPropertyData {
    /// Inner type
    pub inner_type: Box<UsmapPropertyData>,
    /// Value type
    pub value_type: Box<UsmapPropertyData>,
}

impl UsmapMapPropertyData {
    /// Read a `UsmapMapPropertyData` from an asset
    pub fn new<R: ArchiveReader>(asset: &mut UsmapReader<'_, '_, R>) -> Result<Self, Error> {
        let inner_type = UsmapPropertyData::new(asset)?;
        let value_type = UsmapPropertyData::new(asset)?;

        Ok(UsmapMapPropertyData {
            inner_type: Box::new(inner_type),
            value_type: Box::new(value_type),
        })
    }
}

impl UsmapPropertyDataTrait for UsmapMapPropertyData {
    fn write<W: ArchiveWriter>(&self, asset: &mut UsmapWriter<'_, '_, W>) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::MapProperty as u8)?;
        let mut size = self.inner_type.write(asset)?;
        size += self.value_type.write(asset)?;
        Ok(size + size_of::<u8>())
    }

    fn get_property_type(&self) -> EPropertyType {
        EPropertyType::MapProperty
    }
}
