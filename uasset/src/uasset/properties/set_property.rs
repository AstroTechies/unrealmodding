use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

use super::{Property, array_property::ArrayProperty};

#[derive(Hash, PartialEq, Eq)]
pub struct SetProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub array_type: Option<FName>,
    pub value: Vec<Property>,
    pub removed_items: Vec<Property>
}

impl SetProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, engine_version: i32) -> Result<Self, Error> {
        let (array_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None)
        };

        let removed_items = ArrayProperty::new_no_header(
            asset,
            name.clone(),
            false, 
            length, 
            engine_version,
            false, 
            array_type.clone(), property_guid).map(|e| e.value)?;
        
        let items = ArrayProperty::new_no_header(
            asset,
            name.clone(),
            false, 
            length, 
            engine_version,  
            false, 
            array_type.clone(), 
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
