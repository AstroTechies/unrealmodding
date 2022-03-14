use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, ue4version::VER_UE4_INNER_ARRAY_TAG_INFO}, optional_guid};

use super::Property;

#[derive(Debug, Default)]
pub struct ArrayProperty {
    name: FName,
    property_guid: Option<Guid>,
    array_type: Option<FName>,
    value: Vec<Property>,
}

impl ArrayProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, engine_version: i32, asset: &Asset, serialize_struct_differently: bool) -> Result<Self, Error> {
        let (array_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), Some(cursor.read_property_guid()?)),
            false => (None, None)
        };
        ArrayProperty::new_no_header(name, cursor, include_header, length, engine_version, asset, serialize_struct_differently, array_type, property_guid)
    }

    pub fn new_no_header(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, engine_version: i32, asset: &Asset, serialize_struct_differently: bool, array_type: Option<FName>, property_guid: Option<Guid>) -> Result<Self, Error> {
        let num_entries = cursor.read_i32()?;
        let mut value = None;
        let mut entries = Vec::new();
        let mut name = name;

        let mut struct_length = None;
        let mut full_type = None;
        let mut struct_guid = None;
        
        if (array_type.is_some() && &array_type.unwrap().content == "StructProperty") && serialize_struct_differently {
            if engine_version >= VER_UE4_INNER_ARRAY_TAG_INFO {
                name = asset.read_fname()?;
                if &name.unwrap().content == "None" {
                    return Ok(ArrayProperty::default());
                }

                let this_array_type = asset.read_fname()?;
                if &this_array_type.content == "None" {
                    return Ok(ArrayProperty::default());
                }

                if this_array_type.content != array_type.unwrap().content {
                    return Err(Error::new(ErrorKind::Other, format!("Invalid array type {} vs {}", this_array_type.content, array_type.content)));
                }

                struct_length = Some(cursor.read_i64()?);
                full_type = Some(asset.read_fname()?);

                let mut guid = [0u8; 16];
                cursor.read_exact(&mut guid)?;
                struct_guid = Some(guid);
                cursor.read_property_guid()?;
            }

                
            // todo: dummy struct
            for i in 0..num_entries {
                let data = Property::from_type(cursor, asset, &full_type?, name, false, struct_length, 0)?;
                if let Some(data) = data {
                    entries.push(data);
                }
            }
        } else {
            if num_entries > 0 {
                let size_est_1 = length / num_entries as i64;
                let size_est_2 = (length - 4) / num_entries as i64;

                for i in 0..num_entries {
                    let entry = Property::from_type(cursor, asset, &full_type?, FName::new(i.to_string(), i32::MIN), name, false, size_est_1, size_est_2)?;
                    if let Some(entry) = entry {
                        entries.push(entry);
                    }
                }
            }
        }

        Ok(ArrayProperty {
            name,
            property_guid,
            array_type,
            value: entries
        })
    }
}