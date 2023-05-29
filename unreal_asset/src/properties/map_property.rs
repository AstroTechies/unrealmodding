//! Map property

use std::hash::Hash;

use byteorder::LE;
use unreal_asset_proc_macro::FNameContainer;

use crate::containers::indexed_map::IndexedMap;
use crate::error::Error;
use crate::properties::{struct_property::StructProperty, Property, PropertyTrait};
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{
    fname::{FName, ToSerializedName},
    Guid,
};
use crate::unversioned::{
    ancestry::Ancestry,
    properties::{UsmapPropertyData, UsmapPropertyDataTrait},
};
use crate::{cast, impl_property_data_trait};

/// Map property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq)]
pub struct MapProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Key type
    pub key_type: FName,
    /// Value type
    pub value_type: FName,
    /// Map values
    pub value: IndexedMap<Property, Property>,
    /// Keys to be removed from the map when the engine loads the property
    pub keys_to_remove: Option<Vec<Property>>,
}
impl_property_data_trait!(MapProperty);

#[allow(clippy::derived_hash_with_manual_eq)]
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
    /// Map type_name to a `Property` and read it from an asset
    fn map_type_to_class<Reader: ArchiveReader>(
        asset: &mut Reader,
        type_name: FName,
        name: FName,
        ancestry: &Ancestry,
        length: i64,
        include_header: bool,
        is_key: bool,
    ) -> Result<Property, Error> {
        let new_ancestry = ancestry.with_parent(name.clone());
        match type_name.get_content().as_str() {
            "StructProperty" => {
                let mut struct_type = None;

                if let Some(map_data) = asset
                    .get_mappings()
                    .and_then(|e| e.get_property(&name, ancestry))
                    .and_then(|e| cast!(UsmapPropertyData, UsmapMapPropertyData, &e.property_data))
                {
                    match (
                        is_key,
                        map_data.inner_type.as_ref(),
                        map_data.value_type.as_ref(),
                    ) {
                        (true, UsmapPropertyData::UsmapStructPropertyData(inner_type), _) => {
                            struct_type = Some(FName::new_dummy(inner_type.struct_type.clone(), 0));
                        }
                        (false, _, UsmapPropertyData::UsmapStructPropertyData(value_type)) => {
                            struct_type = Some(FName::new_dummy(value_type.struct_type.clone(), 0))
                        }
                        _ => {}
                    }
                }

                let struct_type = struct_type
                    .or_else(|| match is_key {
                        true => asset
                            .get_map_key_override()
                            .get_by_key(&name.get_content())
                            .map(|s| FName::new_dummy(s.to_owned(), 0)),
                        false => asset
                            .get_map_value_override()
                            .get_by_key(&name.get_content())
                            .map(|s| FName::new_dummy(s.to_owned(), 0)),
                    })
                    .unwrap_or_else(|| FName::from_slice("Generic"));

                Ok(StructProperty::custom_header(
                    asset,
                    name,
                    new_ancestry,
                    1,
                    0,
                    Some(struct_type),
                    None,
                    None,
                )?
                .into())
            }
            _ => Property::from_type(
                asset,
                &type_name,
                name,
                new_ancestry,
                include_header,
                length,
                0,
                0,
                false,
            ),
        }
    }

    /// Read a `MapProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let mut type_1 = None;
        let mut type_2 = None;
        let mut property_guid = None;

        if include_header && !asset.has_unversioned_properties() {
            type_1 = Some(asset.read_fname()?);
            type_2 = Some(asset.read_fname()?);
            property_guid = asset.read_property_guid()?;
        }

        if type_1.is_none() && type_2.is_none() {
            if let Some(property) = asset
                .get_mappings()
                .and_then(|e| e.get_property(&name, &ancestry))
                .and_then(|e| cast!(UsmapPropertyData, UsmapMapPropertyData, &e.property_data))
            {
                type_1 = Some(FName::from_slice(
                    &property.inner_type.get_property_type().to_string(),
                ));
                type_2 = Some(FName::from_slice(
                    &property.value_type.get_property_type().to_string(),
                ));
            }
        }

        let num_keys_to_remove = asset.read_i32::<LE>()?;
        let mut keys_to_remove = None;

        let type_1 = type_1.ok_or_else(|| Error::invalid_file("No type1".to_string()))?;
        let type_2 = type_2.ok_or_else(|| Error::invalid_file("No type2".to_string()))?;

        for _ in 0..num_keys_to_remove as usize {
            let mut vec = Vec::with_capacity(num_keys_to_remove as usize);
            vec.push(MapProperty::map_type_to_class(
                asset,
                type_1.clone(),
                name.clone(),
                &ancestry,
                0,
                false,
                true,
            )?);
            keys_to_remove = Some(vec);
        }

        let num_entries = asset.read_i32::<LE>()?;
        let mut values: IndexedMap<Property, Property> = IndexedMap::new();

        for _ in 0..num_entries {
            let key = MapProperty::map_type_to_class(
                asset,
                type_1.clone(),
                name.clone(),
                &ancestry,
                0,
                false,
                true,
            )?;
            let value = MapProperty::map_type_to_class(
                asset,
                type_2.clone(),
                name.clone(),
                &ancestry,
                0,
                false,
                false,
            )?;
            values.insert(key, value);
        }

        Ok(MapProperty {
            name,
            ancestry,
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
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        if include_header && !asset.has_unversioned_properties() {
            if let Some((_, key, value)) = self.value.iter().next() {
                let key_name = key.to_serialized_name();
                let value_name = value.to_serialized_name();

                let mut name_map = asset.get_name_map();
                let mut name_map = name_map.get_mut();

                let key = name_map.add_fname(&key_name);
                let value = name_map.add_fname(&value_name);
                drop(name_map);

                asset.write_fname(&key)?;
                asset.write_fname(&value)?;
            } else {
                asset.write_fname(&self.key_type)?;
                asset.write_fname(&self.value_type)?;
            }
            asset.write_property_guid(&self.property_guid)?;
        }

        let begin = asset.position();
        asset.write_i32::<LE>(match self.keys_to_remove {
            Some(ref e) => e.len(),
            None => 0,
        } as i32)?;

        if let Some(ref keys_to_remove) = self.keys_to_remove {
            for key in keys_to_remove {
                key.write(asset, false)?;
            }
        }

        asset.write_i32::<LE>(self.value.len() as i32)?;

        for (_, key, value) in &self.value {
            key.write(asset, false)?;
            value.write(asset, false)?;
        }

        Ok((asset.position() - begin) as usize)
    }
}
