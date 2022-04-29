use std::io::{Cursor, Read, Write};

use crate::error::{Error, PropertyError};
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait,
    {
        ue4version::{VER_UE4_SERIALIZE_RICH_CURVE_KEY, VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG},
        unreal_types::{FName, Guid},
        Asset,
    },
};

use super::Property;

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct StructProperty {
    pub name: FName,
    pub struct_type: Option<FName>,
    pub struct_guid: Option<Guid>,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub serialize_none: bool,
    pub value: Vec<Property>,
}
impl_property_data_trait!(StructProperty);

impl StructProperty {
    pub fn dummy(name: FName, struct_type: FName, struct_guid: Option<Guid>) -> Self {
        StructProperty {
            name,
            struct_type: Some(struct_type),
            struct_guid,
            property_guid: None,
            duplication_index: 0,
            serialize_none: true,
            value: Vec::new(),
        }
    }

    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        length: i64,
        _duplication_index: i32,
        engine_version: i32,
    ) -> Result<Self, Error> {
        let mut struct_type = None;
        let mut struct_guid = None;
        let mut property_guid = None;

        if include_header {
            struct_type = Some(asset.read_fname()?);
            if engine_version >= VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG {
                let mut guid = [0u8; 16];
                asset.cursor.read_exact(&mut guid)?;
                struct_guid = Some(guid);
            }
            property_guid = asset.read_property_guid()?;
        }

        StructProperty::custom_header(
            asset,
            name,
            length,
            0,
            struct_type,
            struct_guid,
            property_guid,
        )
    }

    pub fn custom_header(
        asset: &mut Asset,
        name: FName,
        length: i64,
        duplication_index: i32,
        struct_type: Option<FName>,
        struct_guid: Option<[u8; 16]>,
        property_guid: Option<[u8; 16]>,
    ) -> Result<Self, Error> {
        let mut custom_serialization = match struct_type {
            Some(ref e) => Property::has_custom_serialization(&e.content),
            None => false,
        };

        if let Some(ref e) = struct_type {
            if e.content.as_str() == "RichCurveKey"
                && asset.engine_version < VER_UE4_SERIALIZE_RICH_CURVE_KEY
            {
                custom_serialization = false;
            }
        }

        if length == 0 {
            return Ok(StructProperty {
                name,
                struct_type,
                struct_guid,
                property_guid,
                duplication_index,
                serialize_none: false,
                value: Vec::new(),
            });
        }

        if custom_serialization {
            let property = Property::from_type(
                asset,
                struct_type.as_ref().unwrap(),
                name.clone(),
                false,
                0,
                0,
                0,
            )?;
            let value = vec![property];

            return Ok(StructProperty {
                name,
                struct_type,
                struct_guid,
                property_guid,
                duplication_index,
                serialize_none: true,
                value,
            });
        } else {
            let mut values = Vec::new();
            let mut property = Property::new(asset, true)?;
            while property.is_some() {
                values.push(property.unwrap());
                property = Property::new(asset, true)?;
            }

            return Ok(StructProperty {
                name,
                struct_type,
                struct_guid,
                property_guid,
                duplication_index,
                serialize_none: true,
                value: values,
            });
        }
    }

    pub fn write_with_type(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
        struct_type: Option<FName>,
    ) -> Result<usize, Error> {
        if include_header {
            asset.write_fname(
                cursor,
                struct_type.as_ref().ok_or(PropertyError::headerless())?,
            )?;
            if asset.engine_version >= VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG {
                cursor.write(&self.struct_guid.ok_or(PropertyError::headerless())?)?;
            }
            asset.write_property_guid(cursor, &self.property_guid)?;
        }

        let mut has_custom_serialization = match struct_type {
            Some(ref e) => Property::has_custom_serialization(&e.content),
            None => false,
        };

        if (struct_type.is_some()
            && struct_type.as_ref().unwrap().content.as_str() == "RichCurveKey")
            && asset.engine_version < VER_UE4_SERIALIZE_RICH_CURVE_KEY
        {
            has_custom_serialization = false;
        }

        if has_custom_serialization {
            if self.value.len() != 1 {
                return Err(PropertyError::invalid_struct(format!(
                    "Structs with type {} must have exactly 1 entry",
                    struct_type
                        .as_ref()
                        .map(|e| e.content.to_owned())
                        .unwrap_or("Generic".to_string())
                ))
                .into());
            }
            return self.value[0].write(asset, cursor, false);
        } else if self.value.len() == 0 && !self.serialize_none {
            return Ok(0);
        } else {
            let begin = cursor.position();
            for entry in &self.value {
                Property::write(entry, asset, cursor, true)?;
            }
            asset.write_fname(cursor, &FName::from_slice("None"))?;
            return Ok((cursor.position() - begin) as usize);
        }
    }
}

impl PropertyTrait for StructProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        self.write_with_type(asset, cursor, include_header, self.struct_type.clone())
    }
}
