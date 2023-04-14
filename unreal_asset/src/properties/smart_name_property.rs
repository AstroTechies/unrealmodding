//! Smart name property

use byteorder::LittleEndian;

use crate::custom_version::FAnimPhysObjectVersion;
use crate::error::PropertyError;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::{FName, Guid};
use crate::Error;

/// Smart name property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct SmartNameProperty {
    /// Name
    pub name: FName,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Display name
    pub display_name: FName,
    /// Smart name id
    pub smart_name_id: Option<u16>,
    /// Temporary guid
    pub temp_guid: Option<Guid>,
}
impl_property_data_trait!(SmartNameProperty);

impl SmartNameProperty {
    /// Read a `SmartNameProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let display_name = asset.read_fname()?;

        let mut smart_name_id = None;
        let mut temp_guid = None;

        let custom_version = asset.get_custom_version::<FAnimPhysObjectVersion>().version;

        if custom_version < FAnimPhysObjectVersion::RemoveUIDFromSmartNameSerialize as i32 {
            smart_name_id = Some(asset.read_u16::<LittleEndian>()?);
        }
        if custom_version < FAnimPhysObjectVersion::SmartNameRefactorForDeterministicCooking as i32
        {
            let mut guid = [0u8; 16];
            asset.read_exact(&mut guid)?;
            temp_guid = Some(guid);
        }

        Ok(SmartNameProperty {
            name,
            property_guid,
            duplication_index,
            display_name,
            smart_name_id,
            temp_guid,
        })
    }
}

impl PropertyTrait for SmartNameProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let begin = asset.position();

        asset.write_fname(&self.display_name)?;

        let custom_version = asset.get_custom_version::<FAnimPhysObjectVersion>().version;
        if custom_version < FAnimPhysObjectVersion::RemoveUIDFromSmartNameSerialize as i32 {
            asset.write_u16::<LittleEndian>(
                self.smart_name_id
                    .ok_or_else(|| PropertyError::property_field_none("smart_name_id", "u16"))?,
            )?;
        }
        if custom_version < FAnimPhysObjectVersion::SmartNameRefactorForDeterministicCooking as i32
        {
            asset.write_all(
                &self
                    .temp_guid
                    .ok_or_else(|| PropertyError::property_field_none("temp_guid", "String"))?,
            )?;
        }
        Ok((asset.position() - begin) as usize)
    }
}
