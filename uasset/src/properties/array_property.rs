use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{Error, PropertyError};
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::ue4version::{
    VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG, VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG,
};
use crate::unreal_types::{default_guid, ToFName};
use crate::{
    impl_property_data_trait,
    ue4version::VER_UE4_INNER_ARRAY_TAG_INFO,
    unreal_types::{FName, Guid},
    Asset,
};

use super::{struct_property::StructProperty, Property};

#[derive(Default, Hash, PartialEq, Eq)]
pub struct ArrayProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub array_type: Option<FName>,
    pub value: Vec<Property>,

    dummy_property: Option<StructProperty>,
}
impl_property_data_trait!(ArrayProperty);

impl ArrayProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        length: i64,
        duplication_index: i32,
        engine_version: i32,
        serialize_struct_differently: bool,
    ) -> Result<Self, Error> {
        let (array_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None),
        };
        ArrayProperty::new_no_header(
            asset,
            name,
            include_header,
            length,
            duplication_index,
            engine_version,
            serialize_struct_differently,
            array_type,
            property_guid,
        )
    }

    pub fn from_arr(name: FName, array_type: Option<FName>, value: Vec<Property>) -> Self {
        ArrayProperty {
            name,
            property_guid: None,
            array_type,
            value,
            duplication_index: 0,
            dummy_property: None,
        }
    }

    pub fn new_no_header(
        asset: &mut Asset,
        name: FName,
        _include_header: bool,
        length: i64,
        duplication_index: i32,
        engine_version: i32,
        serialize_struct_differently: bool,
        array_type: Option<FName>,
        property_guid: Option<Guid>,
    ) -> Result<Self, Error> {
        let _cursor = &mut asset.cursor;
        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let mut entries = Vec::new();
        let mut name = name;

        let mut struct_length = 1;
        let mut struct_guid = None;

        let mut dummy_struct = None;
        if (array_type.is_some()
            && array_type.as_ref().unwrap().content.as_str() == "StructProperty")
            && serialize_struct_differently
        {
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
                    return Err(Error::invalid_file(format!(
                        "Invalid array type {} vs {}",
                        this_array_type.content,
                        array_type.as_ref().unwrap().content
                    )));
                }

                struct_length = asset.cursor.read_i64::<LittleEndian>()?;
                full_type = asset.read_fname()?;

                let mut guid = [0u8; 16];
                asset.cursor.read_exact(&mut guid)?;
                struct_guid = Some(guid);
                asset.read_property_guid()?;
            }

            if num_entries == 0 {
                dummy_struct = Some(StructProperty::dummy(
                    name.clone(),
                    full_type.clone(),
                    struct_guid,
                ));
            }
            for _i in 0..num_entries {
                let data = StructProperty::custom_header(
                    asset,
                    name.clone(),
                    struct_length,
                    0,
                    Some(full_type.clone()),
                    struct_guid,
                    None,
                )?;
                entries.push(data.into());
            }
        } else {
            if num_entries > 0 {
                let size_est_1 = length / num_entries as i64;
                let size_est_2 = (length - 4) / num_entries as i64;
                let array_type = array_type
                    .as_ref()
                    .ok_or(Error::invalid_file("Unknown array type".to_string()))?;
                for i in 0..num_entries {
                    let entry = Property::from_type(
                        asset,
                        array_type,
                        FName::new(i.to_string(), i32::MIN),
                        false,
                        size_est_1,
                        size_est_2,
                        0,
                    )?;
                    entries.push(entry);
                }
            }
        }

        Ok(ArrayProperty {
            name,
            property_guid,
            duplication_index,
            array_type,
            dummy_property: dummy_struct,
            value: entries,
        })
    }

    pub fn write_full(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
        serialize_structs_differently: bool,
    ) -> Result<usize, Error> {
        let array_type = match self.value.len() > 0 {
            true => Some(self.value[0].to_fname()),
            false => self.array_type.clone(),
        };

        if include_header {
            asset.write_fname(
                cursor,
                array_type.as_ref().ok_or(PropertyError::headerless())?,
            )?;
            asset.write_property_guid(cursor, &self.property_guid)?;
        }

        let begin = cursor.position();
        cursor.write_i32::<LittleEndian>(self.value.len() as i32)?;

        if (array_type.is_some()
            && array_type.as_ref().unwrap().content.as_str() == "StructProperty")
            && serialize_structs_differently
        {
            let property: &StructProperty = match self.value.len() > 0 {
                true => match &self.value[0] {
                    Property::StructProperty(ref e) => Ok(e),
                    _ => Err(PropertyError::invalid_array(format!(
                        "expected StructProperty got {}",
                        self.value[0].to_fname().content
                    ))),
                },
                false => match self.dummy_property {
                    Some(ref e) => Ok(e),
                    None => Err(PropertyError::invalid_array(
                        "Empty array with no dummy struct. Cannot serialize".to_string(),
                    )),
                },
            }?;

            let mut length_loc = -1;
            if asset.engine_version >= VER_UE4_INNER_ARRAY_TAG_INFO {
                asset.write_fname(cursor, &property.name)?;
                asset.write_fname(cursor, &FName::from_slice("StructProperty"))?;
                length_loc = cursor.position() as i32;
                cursor.write_i64::<LittleEndian>(0)?;
                asset.write_fname(
                    cursor,
                    property
                        .struct_type
                        .as_ref()
                        .ok_or(PropertyError::property_field_none("struct_type", "FName"))?,
                )?;
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
                    _ => Err(PropertyError::invalid_array(format!(
                        "expected StructProperty got {}",
                        property.to_fname().content
                    ))),
                }?;
                struct_property.write(asset, cursor, false)?;
            }

            if asset.engine_version >= VER_UE4_INNER_ARRAY_TAG_INFO {
                let full_len = cursor.position() as i32 - length_loc;
                let new_loc = cursor.position() as i32;
                cursor.seek(SeekFrom::Start(length_loc as u64))?;
                let length = full_len
                    - 32
                    - match include_header {
                        true => 1,
                        false => 0,
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
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        self.write_full(asset, cursor, include_header, true)
    }
}
