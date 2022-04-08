use std::io::{Cursor, ErrorKind, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::uasset::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, custom_version::FAnimPhysObjectVersion}, optional_guid, optional_guid_write};
use crate::uasset::error::PropertyError;
use crate::uasset::properties::PropertyTrait;

#[derive(Hash, PartialEq, Eq)]
pub struct SmartNameProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    
    pub display_name: FName,
    pub smart_name_id: Option<u16>,
    pub temp_guid: Option<Guid>
}

impl SmartNameProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let display_name = asset.read_fname()?;

        let mut smart_name_id = None;
        let mut temp_guid = None;

        let custom_version = asset.get_custom_version::<FAnimPhysObjectVersion>().version;

        if custom_version < FAnimPhysObjectVersion::RemoveUIDFromSmartNameSerialize as i32 {
            smart_name_id = Some(asset.cursor.read_u16::<LittleEndian>()?);
        }
        if custom_version < FAnimPhysObjectVersion::SmartNameRefactorForDeterministicCooking as i32 {
            let mut guid = [0u8; 16];
            asset.cursor.read_exact(&mut guid);
            temp_guid = Some(guid);
        }

        Ok(SmartNameProperty {
            name,
            property_guid,
            display_name,
            smart_name_id,
            temp_guid
        })
    }
}

impl PropertyTrait for SmartNameProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        let begin = cursor.position();

        asset.write_fname(cursor, &self.display_name)?;

        let custom_version = asset.get_custom_version::<FAnimPhysObjectVersion>().version;
        if custom_version < FAnimPhysObjectVersion::RemoveUIDFromSmartNameSerialize as i32 {
            cursor.write_u16::<LittleEndian>(self.smart_name_id.ok_or(PropertyError::property_field_none("smart_name_id", "u16"))?)?;
        }
        if custom_version < FAnimPhysObjectVersion::SmartNameRefactorForDeterministicCooking as i32 {
            cursor.write(&self.temp_guid.ok_or(PropertyError::property_field_none("temp_guid", "String"))?)?;
        }
        Ok((cursor.position() - begin) as usize)
    }
}
