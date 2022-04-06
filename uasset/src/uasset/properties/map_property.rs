use core::num;
use std::{io::{Cursor, ErrorKind}, collections::HashMap, hash::Hash};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};
use crate::uasset::error::Error;
use super::{Property, struct_property::StructProperty};

#[derive(PartialEq, Eq)]
pub struct MapProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub key_type: FName,
    pub value_type: FName,
    pub value: HashMap<Property, Property>
}

impl Hash for MapProperty {
    //todo: probably do something with map
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.property_guid.hash(state);
        self.key_type.hash(state);
        self.value_type.hash(state);
    }
}

impl MapProperty {

    fn map_type_to_class(asset: &mut Asset, type_name: FName, name: FName, length: i64, include_header: bool, is_key: bool) -> Result<Property, Error> {
        match type_name.content.as_str() {
            "StructProperty" => {
                let struct_type = match is_key {
                    true => asset.map_key_override.get(&name.content).map(|s| s.to_owned()),
                    false => asset.map_value_override.get(&name.content).map(|s| s.to_owned())
                }.unwrap_or(String::from("Generic"));

                Property::from_type(asset, &FName::new(struct_type.to_string(), 0), name, false, 1, 0)
            },
            _ => {
                Property::from_type(asset, &type_name, name, include_header, length, 0)
            }
        }
    }

    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let mut type_1 = None;
        let mut type_2 = None;
        let mut property_guid = None;

        if include_header {
            type_1 = Some(asset.read_fname()?);
            type_2 = Some(asset.read_fname()?);
            property_guid = asset.read_property_guid()?;
        }

        let num_keys_to_remove = asset.cursor.read_i32::<LittleEndian>()?;
        let mut keys_to_remove = Vec::with_capacity(num_keys_to_remove as usize);
        
        let type_1 = type_1.ok_or(Error::invalid_file("No type1".to_string()))?;
        let type_2 = type_2.ok_or(Error::invalid_file("No type2".to_string()))?;

        for i in 0..num_keys_to_remove as usize {
            keys_to_remove.push(MapProperty::map_type_to_class(asset, type_1.clone(), name.clone(), 0, false, true)?);
        }

        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let mut values = HashMap::new();

        for i in 0..num_entries {
            let key = MapProperty::map_type_to_class(asset, type_1.clone(), name.clone(), 0, false, true)?;
            let value = MapProperty::map_type_to_class(asset, type_2.clone(), name.clone(), 0, false, false)?;

            values.insert(key, value);
        }

        Ok(MapProperty {
            name,
            property_guid: property_guid,
            key_type: type_1,
            value_type: type_2,
            value: values
        })
    }
}
