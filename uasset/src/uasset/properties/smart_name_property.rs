use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, custom_version::FAnimPhysObjectVersion}, optional_guid};

pub struct SmartNameProperty {
    name: FName,
    property_guid: Option<Guid>,
    
    display_name: FName,
    smart_name_id: Option<u16>,
    temp_guid: Option<Guid>
}

impl SmartNameProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        let display_name = asset.read_fname()?;

        let mut smart_name_id = None;
        let mut temp_guid = None;

        let custom_version: FAnimPhysObjectVersion = asset.get_custom_version("FAnimPhysObjectVersion")?.into()?;
        if custom_version < FAnimPhysObjectVersion::RemoveUIDFromSmartNameSerialize {
            smart_name_id = Some(cursor.read_u16()?);
        }
        if custom_version < FAnimPhysObjectVersion::SmartNameRefactorForDeterministicCooking {
            let mut guid = [0u8; 16];
            cursor.read_exact(&mut guid);
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