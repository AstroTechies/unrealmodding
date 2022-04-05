use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, ue4version::VER_UE4_ADDED_SOFT_OBJECT_PATH}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct SoftPathProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub asset_path_name: Option<FName>,
    pub sub_path: Option<String>,
    pub path: Option<String>
}

impl SoftPathProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        
        let mut path = None;
        let mut asset_path_name = None;
        let mut sub_path = None;

        if asset.engine_version < VER_UE4_ADDED_SOFT_OBJECT_PATH {
            path = Some(asset.cursor.read_string()?);
        } else {
            asset_path_name = Some(asset.read_fname()?);
            sub_path = Some(asset.cursor.read_string()?);
        }

        Ok(SoftPathProperty {
            name,
            property_guid,
            asset_path_name,
            sub_path,
            path
        })
    }
}
