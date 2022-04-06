use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, custom_version::FAnimPhysObjectVersion}, optional_guid};

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
