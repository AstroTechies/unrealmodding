use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

use super::{Property, array_property::ArrayProperty};

pub struct SetProperty {
    name: FName,
    property_guid: Option<Guid>,
    array_type: Option<FName>,
    value: Vec<Property>,
    removed_items: Vec<Property>
}

impl SetProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, engine_version: i32, asset: &Asset) -> Result<Self, Error> {
        let (array_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), Some(cursor.read_property_guid()?)),
            false => (None, None)
        };

        let removed_items = ArrayProperty::new_no_header(
            name,
            cursor, 
            false, 
            length, 
            engine_version, 
            asset, 
            false, 
            array_type, property_guid).map(|e| e.value)?;
        
        let items = ArrayProperty::new_no_header(
            name,
            cursor, 
            false, 
            length, 
            engine_version, 
            asset, 
            false, 
            array_type, 
            property_guid).map(|e| e.value)?;
        
        Ok(SetProperty {
            name,
            property_guid,
            array_type,
            value: items,
            removed_items
        })
    }
}