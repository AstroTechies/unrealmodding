//! Shallow property

use crate::{
    error::Error, reader::archive_writer::ArchiveWriter, unversioned::usmap_writer::UsmapWriter,
};

use super::{EPropertyType, UsmapPropertyDataTrait};

/// Shallow property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapShallowPropertyData {
    /// Property type
    pub property_type: EPropertyType,
}

impl UsmapPropertyDataTrait for UsmapShallowPropertyData {
    fn write<'parent_writer, 'asset, W: ArchiveWriter>(
        &self,
        asset: &mut UsmapWriter<'parent_writer, 'asset, W>,
    ) -> Result<usize, Error> {
        asset.write_u8(self.property_type as u8)?;
        Ok(0)
    }

    fn get_property_type(&self) -> EPropertyType {
        self.property_type
    }
}
