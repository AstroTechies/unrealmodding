//! Raw struct property

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    types::{FName, Guid},
};

use super::PropertyTrait;

/// Raw struct property
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawStructProperty {
    /// Name
    pub name: FName,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Raw data
    pub value: Vec<u8>,
}
impl_property_data_trait!(RawStructProperty);

impl RawStructProperty {
    /// Read a `RawStructProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
        length: i64,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let mut value = vec![0u8; length as usize];
        asset.read_exact(&mut value)?;

        Ok(RawStructProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for RawStructProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        asset.write_all(&self.value)?;

        Ok((asset.position() - begin) as usize)
    }
}
