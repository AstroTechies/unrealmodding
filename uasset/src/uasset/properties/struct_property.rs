use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, ue4version::{VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG, VER_UE4_SERIALIZE_RICH_CURVE_KEY}, Asset}, optional_guid};

use super::Property;

#[derive(Hash, PartialEq, Eq)]
pub struct StructProperty {
    pub name: FName,
    pub struct_type: Option<FName>,
    pub struct_guid: Option<Guid>,
    pub property_guid: Option<Guid>,
    pub serialize_none: bool,
    pub value: Vec<Property>
}

impl StructProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, engine_version: i32) -> Result<Self, Error> {
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

        StructProperty::custom_header(asset, name, length, struct_type, struct_guid, property_guid)
    }

    pub fn custom_header(asset: &mut Asset, name: FName, length: i64, struct_type: Option<FName>, struct_guid: Option<[u8; 16]>, property_guid: Option<[u8; 16]>) -> Result<Self, Error> {
        let mut custom_serialization = match struct_type {
            Some(ref e) => Property::has_custom_serialization(&e.content),
            None => false
        };

        if let Some(ref e) = struct_type {
            if e.content.as_str() == "RichCurveKey" && asset.engine_version < VER_UE4_SERIALIZE_RICH_CURVE_KEY {
                custom_serialization = false;
            }
        }

        if length == 0 {
            return Ok(StructProperty {
                name,
                struct_type,
                struct_guid,
                property_guid,
                serialize_none: false,
                value: Vec::new()
            });
        }

        if custom_serialization {
            let property = Property::from_type(asset, struct_type.as_ref().unwrap(), name.clone(), false, 0, 0)?;
            let value = vec![property];

            return Ok(StructProperty {
                name,
                struct_type,
                struct_guid,
                property_guid,
                serialize_none: true,
                value
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
                serialize_none: true,
                value: values
            });
        }
    }
}
