use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, ue4version::{VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG, VER_UE4_SERIALIZE_RICH_CURVE_KEY}}, optional_guid};

use super::Property;

pub struct StructProperty {
    name: FName,
    struct_type: Option<FName>,
    struct_guid: Option<Guid>,
    property_guid: Option<Guid>,
    serialize_none: bool,
    value: Vec<Property>
}

impl StructProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, engine_version: i32, asset: &Asset) -> Result<Self, Error> {
        let mut struct_type = None;
        let mut struct_guid = None;
        let mut property_guid = None;
        
        if include_header {
            struct_type = Some(asset.read_fname()?);
            if engine_version >= VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG {
                let mut guid = [0u8; 16];
                cursor.read_exact(&mut guid)?;
                struct_guid = Some(guid);
            }
            property_guid = Some(cursor.read_property_guid()?);
        }

        let mut custom_serialization = match struct_type {
            Some(ref e) => Property::has_custom_serialization(e.content),
            None => false
        };

        if let Some(e) = struct_type {
            if &e.content == "RichCurveKey" && engine_version < VER_UE4_SERIALIZE_RICH_CURVE_KEY {
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
            let property = Property::from_type(cursor, asset, struct_type.unwrap(), name, false, 0, 0)?;
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
            let mut property = Property::new(cursor, asset, true)?;
            while property.is_some() {
                values.push(property.unwrap());
                property = Property::new(cursor, asset, true)?;
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