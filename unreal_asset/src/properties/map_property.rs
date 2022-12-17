use std::{collections::HashMap, hash::Hash};

use byteorder::LittleEndian;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::properties::{struct_property::StructProperty, Property, PropertyTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::ToFName;
use crate::unreal_types::{FName, Guid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub key_type: FName,
    pub value_type: FName,
    pub value: HashMap<Property, Property>,
    pub keys_to_remove: Option<Vec<Property>>,
}
impl_property_data_trait!(MapProperty);

#[allow(clippy::derive_hash_xor_eq)]
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
    fn map_type_to_class<Reader: AssetReader>(
        asset: &mut Reader,
        type_name: FName,
        name: FName,
        length: i64,
        include_header: bool,
        is_key: bool,
    ) -> Result<Property, Error> {
        match type_name.content.as_str() {
            "StructProperty" => {
                let struct_type = match is_key {
                    true => asset
                        .get_map_key_override()
                        .get(&name.content)
                        .map(|s| s.to_owned()),
                    false => asset
                        .get_map_value_override()
                        .get(&name.content)
                        .map(|s| s.to_owned()),
                }
                .unwrap_or_else(|| String::from("Generic"));
                Ok(StructProperty::custom_header(
                    asset,
                    name,
                    1,
                    0,
                    Some(FName::from_slice(&struct_type)),
                    None,
                    None,
                )?
                .into())
            }
            _ => Property::from_type(asset, &type_name, name, include_header, length, 0, 0),
        }
    }

    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let mut type_1 = None;
        let mut type_2 = None;
        let mut property_guid = None;

        if include_header {
            type_1 = Some(asset.read_fname()?);
            type_2 = Some(asset.read_fname()?);
            property_guid = asset.read_property_guid()?;
        }

        let num_keys_to_remove = asset.read_i32::<LittleEndian>()?;
        let mut keys_to_remove = None;

        let type_1 = type_1.ok_or_else(|| Error::invalid_file("No type1".to_string()))?;
        let type_2 = type_2.ok_or_else(|| Error::invalid_file("No type2".to_string()))?;

        for _ in 0..num_keys_to_remove as usize {
            let mut vec = Vec::with_capacity(num_keys_to_remove as usize);
            vec.push(MapProperty::map_type_to_class(
                asset,
                type_1.clone(),
                name.clone(),
                0,
                false,
                true,
            )?);
            keys_to_remove = Some(vec);
        }

        let num_entries = asset.read_i32::<LittleEndian>()?;
        let mut values: HashMap<Property, Property> = HashMap::new();

        for _ in 0..num_entries {
            let key = MapProperty::map_type_to_class(
                asset,
                type_1.clone(),
                name.clone(),
                0,
                false,
                true,
            )?;
            let value = MapProperty::map_type_to_class(
                asset,
                type_2.clone(),
                name.clone(),
                0,
                false,
                false,
            )?;
            values.insert(key, value);
        }

        Ok(MapProperty {
            name,
            property_guid,
            duplication_index,
            key_type: type_1,
            value_type: type_2,
            value: values,
            keys_to_remove,
        })
    }
}

impl PropertyTrait for MapProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        if include_header {
            if let Some(key) = self.value.keys().next() {
                asset.write_fname(&key.to_fname())?;
                let value = self.value.values().next().unwrap();
                asset.write_fname(&value.to_fname())?;
            } else {
                asset.write_fname(&self.key_type)?;
                asset.write_fname(&self.value_type)?
            }
            asset.write_property_guid(&self.property_guid)?;
        }

        let begin = asset.position();
        asset.write_i32::<LittleEndian>(match self.keys_to_remove {
            Some(ref e) => e.len(),
            None => 0,
        } as i32)?;

        if let Some(ref keys_to_remove) = self.keys_to_remove {
            for key in keys_to_remove {
                key.write(asset, false)?;
            }
        }

        asset.write_i32::<LittleEndian>(self.value.len() as i32)?;

        for (key, value) in &self.value {
            key.write(asset, false)?;
            value.write(asset, false)?;
        }

        Ok((asset.position() - begin) as usize)
    }
}
