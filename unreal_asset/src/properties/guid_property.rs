use std::io::{Cursor, Write};

use crate::asset_reader::AssetReader;
use crate::asset_writer::AssetWriter;
use crate::error::Error;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    unreal_types::{FName, Guid},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GuidProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Guid,
}
impl_property_data_trait!(GuidProperty);

impl GuidProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let mut value = [0u8; 16];
        asset.read_exact(&mut value)?;
        Ok(GuidProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for GuidProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &Writer,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_all(&self.value)?;
        Ok(16)
    }
}
