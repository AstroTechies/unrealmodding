use std::mem::size_of;

use crate::error::{Error, PropertyError};
use crate::impl_property_data_trait;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::{FName, Guid};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct EnumProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub enum_type: Option<FName>,
    pub value: FName,
}
impl_property_data_trait!(EnumProperty);

impl EnumProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let (enum_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None),
        };
        let value = asset.read_fname()?;

        Ok(EnumProperty {
            name,
            property_guid,
            duplication_index,
            enum_type,
            value,
        })
    }
}

impl PropertyTrait for EnumProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        if include_header {
            asset.write_fname(
                self.enum_type
                    .as_ref()
                    .ok_or_else(PropertyError::headerless)?,
            )?;
            asset.write_property_guid(&self.property_guid)?;
        }
        asset.write_fname(&self.value)?;

        Ok(size_of::<i32>() * 2)
    }
}
