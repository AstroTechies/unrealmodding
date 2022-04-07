use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset, ue4version::VER_UE4_INNER_ARRAY_TAG_INFO}, optional_guid};
use crate::uasset::error::{Error, PropertyError};
use crate::uasset::properties::Property::StructProperty;
use crate::uasset::properties::PropertyTrait;
use crate::uasset::ue4version::{VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG, VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG};
use crate::uasset::unreal_types::default_guid;

use super::{Property, struct_property::StructProperty};

#[derive(Default, Hash, PartialEq, Eq)]
pub struct ArrayProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub array_type: Option<FName>,
    pub value: Vec<Property>,

    dummy_property: Option<StructProperty>
}

impl ArrayProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64, engine_version: i32, serialize_struct_differently: bool) -> Result<Self, Error> {
        let (array_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None)
        };
        ArrayProperty::new_no_header(asset, name, include_header, length, engine_version, serialize_struct_differently, array_type, property_guid)
    }

    pub fn new_no_header(asset: &mut Asset, name: FName, include_header: bool, length: i64, engine_version: i32, serialize_struct_differently: bool, array_type: Option<FName>, property_guid: Option<Guid>) -> Result<Self, Error> {
        let mut cursor = &mut asset.cursor;
        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let mut entries = Vec::new();
        let mut name = name;

        let mut struct_length = 1;
        let mut struct_guid = None;

        let mut dummy_struct = None;
        if (array_type.is_some() && array_type.as_ref().unwrap().content.as_str() == "StructProperty") && serialize_struct_differently {
            let mut full_type = FName::new(String::from("Generic"), 0);
            if engine_version >= VER_UE4_INNER_ARRAY_TAG_INFO {
                name = asset.read_fname()?;
                if &name.content == "None" {
                    return Ok(ArrayProperty::default());
                }

                let this_array_type = asset.read_fname()?;
                if &this_array_type.content == "None" {
                    return Ok(ArrayProperty::default());
                }

                if this_array_type.content != array_type.as_ref().unwrap().content.as_str() {
                    return Err(Error::invalid_file(format!("Invalid array type {} vs {}", this_array_type.content, array_type.as_ref().unwrap().content)));
                }

                struct_length = asset.cursor.read_i64::<LittleEndian>()?;
                full_type = asset.read_fname()?;

                let mut guid = [0u8; 16];
                asset.cursor.read_exact(&mut guid)?;
                struct_guid = Some(guid);
                asset.read_property_guid()?;
            }


            if num_entries == 0 {
                dummy_struct = Some(StructProperty::dummy(name.clone(), full_type.clone(), struct_guid));
            }
            for i in 0..num_entries {
                let data = StructProperty::custom_header(asset, name.clone(), struct_length, Some(full_type.clone()), struct_guid, None)?;
                entries.push(data.into());
            }
        } else {
            if num_entries > 0 {
                let size_est_1 = length / num_entries as i64;
                let size_est_2 = (length - 4) / num_entries as i64;
                let array_type = array_type.as_ref().ok_or(Error::invalid_file("Unknown array type".to_string()))?;
                for i in 0..num_entries {
                    let entry = Property::from_type(asset, array_type, FName::new(i.to_string(), i32::MIN), false, size_est_1, size_est_2)?;
                    entries.push(entry);
                }
            }
        }

        Ok(ArrayProperty {
            name,
            property_guid,
            array_type,
            dummy_property,
            value: entries
        })
    }

    pub fn write_full(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool, serialize_structs_differently: bool) -> Result<usize, Error> {
        let array_type = match self.value.len() > 0 {
            true => Some(FName::new(self.value[0].to_string(), 0)),
            false => self.array_type.clone()
        };

        if include_header {
            asset.write_fname(cursor, array_type.as_ref().ok_or(PropertyError::headerless().into())?)?;
            asset.write_property_guid(cursor, &self.property_guid)
        }

        let begin = cursor.position();
        cursor.write_i32::<LittleEndian>(self.value.len() as i32)?;

        if (array_type.is_some() && array_type.as_ref().unwrap().content.as_str() == "StructProperty") && serialize_structs_differently {
            let property: &StructProperty = match self.value.len() > 0 {
                true => match &self.value[0] {
                    Property::StructProperty(ref e) => Ok(e),
                    _ => Err(PropertyError::invalid_array(format!("expected StructProperty got {}", self.value[0].to_string())))
                },
                false => match self.dummy_property {
                    Some(ref e) => Ok(e),
                    None => Err(PropertyError::invalid_array("Empty array with no dummy struct. Cannot serialize".to_string()))
                }
            }?;

            let mut length_loc = -1;
            if asset.engine_version >= VER_UE4_INNER_ARRAY_TAG_INFO {
                cursor.write_string(&struct_name)?;
                asset.write_fname(cursor, &FName::from_slice("StructProperty"))?;
                length_loc = cursor.position() as i32;
                cursor.write_i64::<LittleEndian>(0)?;
                asset.write_fname(cursor, &full_type)?;
                if asset.engine_version >= VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG {
                    cursor.write(&property.property_guid.unwrap_or(default_guid()))?;
                }
                if asset.engine_version >= VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG {
                    cursor.write_u8(0)?;
                }
            }

            for property in &self.value {
                let struct_property: &StructProperty = match property {
                    Property::StructProperty(e) => Ok(e),
                    _ => Err(PropertyError::invalid_array(format!("expected StructProperty got {}", property.to_string())))
                }?;
                struct_property.write(asset, cursor, false)?;
            }

            if asset.engine_version >= VER_UE4_INNER_ARRAY_TAG_INFO {
                let full_len = cursor.position() as i32 - length_loc;
                let new_loc = cursor.position() as i32;
                cursor.seek(SeekFrom::Start(length_loc as u64))?;
                let length = full_len - 32 - match include_header {
                    true => 1,
                    false => 0
                };

                cursor.write_i32::<LittleEndian>(length)?;
                cursor.seek(SeekFrom::Start(new_loc as u64))?;
            }
        } else {
            for entry in &self.value {
                entry.write(asset, cursor, false)?;
            }
        }
        Ok((cursor.position() - begin) as usize)
    }
}

impl PropertyTrait for ArrayProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        self.write_full(asset, cursor, include_header, true)
    }
}