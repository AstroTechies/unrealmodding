use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, custom_version::FAnimPhysObjectVersion}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
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

        let custom_version = asset.get_custom_version("FAnimPhysObjectVersion").ok_or(Error::new(ErrorKind::Other, "Unknown custom version"))?;

        if custom_version.version < FAnimPhysObjectVersion::RemoveUIDFromSmartNameSerialize as i32 {
            smart_name_id = Some(cursor.read_u16::<LittleEndian>()?);
        }
        if custom_version.version < FAnimPhysObjectVersion::SmartNameRefactorForDeterministicCooking as i32 {
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