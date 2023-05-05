//! Set property

use std::mem::size_of;

use crate::{
    error::Error,
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
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
    pub fn new<R: ArchiveReader>(asset: &mut UsmapReader<'_, '_, R>) -> Result<Self, Error> {
        let inner_type = UsmapPropertyData::new(asset)?;

        Ok(UsmapSetPropertyData {
            inner_type: Box::new(inner_type),
        })
    }
}

impl UsmapPropertyDataTrait for UsmapSetPropertyData {
    fn write<W: ArchiveWriter>(&self, asset: &mut UsmapWriter<'_, '_, W>) -> Result<usize, Error> {
        asset.write_u8(EPropertyType::SetProperty as u8)?;
        let size = self.inner_type.write(asset)?;
        Ok(size + size_of::<u8>())
    }

    fn get_property_type(&self) -> EPropertyType {
        EPropertyType::SetProperty
    }
}
