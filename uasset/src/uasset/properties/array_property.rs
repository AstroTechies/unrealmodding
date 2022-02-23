use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, ue4version::VER_UE4_INNER_ARRAY_TAG_INFO}, optional_guid};

use super::Property;

#[derive(Debug, Default)]
pub struct ArrayProperty {
    property_guid: Option<Guid>,
    array_type: Option<FName>,
    value: Vec<Property>,
}

impl ArrayProperty {

    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool, engine_version: i32, asset: &Asset, serialize_struct_differently: bool) -> Result<Self, Error> {
        let (array_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), Some(cursor.read_property_guid()?)),
            false => (None, None)
        };

        let num_entries = cursor.read_i32()?;
        let mut value = None;
        if (array_type.is_some() && &array_type.unwrap().content == "StructProperty") && serialize_struct_differently {
            let mut entires = Vec::new();
            let mut name = None;

            let mut struct_length = None;
            let mut full_type = None;
            let mut struct_guid = None;

            if engine_version >= VER_UE4_INNER_ARRAY_TAG_INFO {
                name = Some(asset.read_fname()?);
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

            if num_entries == 0 {
                
            }
        }
    }
}