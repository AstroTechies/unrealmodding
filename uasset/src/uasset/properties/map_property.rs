use core::num;
use std::{io::{Cursor, Error, ErrorKind}, collections::HashMap};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

use super::{Property, struct_property::StructProperty};

pub struct MapProperty {
    name: FName,
    property_guid: Guid,
    key_type: FName,
    value_type: FName,
    value: HashMap<Property, Property>
}

impl MapProperty {

    fn map_type_to_class(type_name: FName, name: FName, cursor: &mut Cursor<Vec<u8>>, length: i32, include_header: bool, is_key: bool, asset: &Asset) -> Result<Property, Error> {
        match type_name.content {
            "StructProperty" => {
                let struct_type = match is_key {
                    true => asset.map_key_override.get(name.content),
                    false => asset.map_value_override.get(name.content)
                }.unwrap_or("Generic");

                let prop = Property::from_type(cursor, asset, struct_type, name, false, 1, 0)?.ok_or(Error::new(ErrorKind::Other, "No such property"))?;
                prop
            },
            _ => {
                let prop = Property::from_type(cursor, asset, type_name, name, include_header, length, 0)?.ok_or(Error::new(ErrorKind::Other, "No such property"))?;
                prop
            }
        }
    }

    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let mut type_1 = None;
        let mut type_2 = None;
        let mut property_guid = None;

        if include_header {
            type_1 = Some(asset.read_fname()?);
            type_2 = Some(asset.read_fname()?);
            property_guid = Some(cursor.read_property_guid()?);
        }

        let num_keys_to_remove = cursor.read_i32::<LittleEndian>()?;
        let keys_to_remove = Vec::with_capacity(num_keys_to_remove);

        let type_1 = type_1?;
        let type_2 = type_2?;

        for i in 0..num_keys_to_remove {
            keys_to_remove[i] = MapProperty::map_type_to_class(type_1, name, cursor, 0, false, true, asset)?;
        }

        let num_entries = cursor.read_i32::<LittleEndian>()?;
        let mut values = HashMap::new();

        for i in 0..num_entries {
            let key = MapProperty::map_type_to_class(type_1, name, cursor, 0, false, true, asset)?;
            let value = MapProperty::map_type_to_class(type_2, name, cursor, 0, false, false, asset)?;

            values.insert(key, value);
        }

        Ok(MapProperty {
            name,
            property_guid: property_guid?,
            key_type: type_1,
            value_type: type_2,
            value: values
        })
    }
}