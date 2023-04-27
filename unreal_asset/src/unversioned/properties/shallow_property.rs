//! Shallow property

use crate::{error::Error, unversioned::usmap_writer::UsmapWriter};

use super::{EPropertyType, UsmapPropertyDataTrait};

/// Shallow property data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UsmapShallowPropertyData {
    /// Property type
    pub property_type: EPropertyType,
}

impl UsmapPropertyDataTrait for UsmapShallowPropertyData {
    fn write<Writer: UsmapWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u8(self.property_type as u8)?;
        Ok(0)
    }

    fn get_property_type(&self) -> EPropertyType {
        self.property_type
    }
}
