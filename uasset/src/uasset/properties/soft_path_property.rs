use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

pub struct SoftObjectPathProperty {
    name: FName,
    property_guid: Option<Guid>,
    asset_path_name: FName,
    sub_path: String,
    path: String
}

impl SoftObjectPathProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        
    }        
}