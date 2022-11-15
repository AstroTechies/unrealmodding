use std::io::SeekFrom;

use byteorder::LittleEndian;

use crate::error::{Error, PropertyError};
use crate::impl_property_data_trait;
use crate::object_version::ObjectVersion;
use crate::properties::{
    struct_property::StructProperty, Property, PropertyDataTrait, PropertyTrait,
};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{default_guid, FName, Guid, ToFName};

#[derive(Default, Clone, Hash, PartialEq, Eq)]
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
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        length: i64,
        duplication_index: i32,
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

    #[allow(clippy::too_many_arguments)]
    pub fn new_no_header<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        _include_header: bool,
        length: i64,
        duplication_index: i32,
        serialize_struct_differently: bool,
        array_type: Option<FName>,
        property_guid: Option<Guid>,
    ) -> Result<Self, Error> {
        let num_entries = asset.read_i32::<LittleEndian>()?;
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
            if asset.get_object_version() >= ObjectVersion::VER_UE4_INNER_ARRAY_TAG_INFO {
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

                struct_length = asset.read_i64::<LittleEndian>()?;
                full_type = asset.read_fname()?;

                let mut guid = [0u8; 16];
                asset.read_exact(&mut guid)?;
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
        } else if num_entries > 0 {
            let size_est_1 = length / num_entries as i64;
            let size_est_2 = (length - 4) / num_entries as i64;
            let array_type = array_type
                .as_ref()
                .ok_or_else(|| Error::invalid_file("Unknown array type".to_string()))?;
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

        Ok(ArrayProperty {
            name,
            property_guid,
            duplication_index,
            array_type,
            dummy_property: dummy_struct,
            value: entries,
        })
    }

    pub fn write_full<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
        serialize_structs_differently: bool,
    ) -> Result<usize, Error> {
        let array_type = match !self.value.is_empty() {
            true => Some(self.value[0].to_fname()),
            false => self.array_type.clone(),
        };

        if include_header {
            asset.write_fname(array_type.as_ref().ok_or_else(PropertyError::headerless)?)?;
            asset.write_property_guid(&self.property_guid)?;
        }

        let begin = asset.position();
        asset.write_i32::<LittleEndian>(self.value.len() as i32)?;

        if (array_type.is_some()
            && array_type.as_ref().unwrap().content.as_str() == "StructProperty")
            && serialize_structs_differently
        {
            let property: &StructProperty = match !self.value.is_empty() {
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
            if asset.get_object_version() >= ObjectVersion::VER_UE4_INNER_ARRAY_TAG_INFO {
                asset.write_fname(&property.name)?;
                asset.write_fname(&FName::from_slice("StructProperty"))?;
                length_loc = asset.position() as i32;
                asset.write_i64::<LittleEndian>(0)?;
                asset.write_fname(
                    property.struct_type.as_ref().ok_or_else(|| {
                        PropertyError::property_field_none("struct_type", "FName")
                    })?,
                )?;
                if asset.get_object_version() >= ObjectVersion::VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG
                {
                    asset.write_all(&property.property_guid.unwrap_or_else(default_guid))?;
                }
                if asset.get_object_version()
                    >= ObjectVersion::VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG
                {
                    asset.write_u8(0)?;
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
                struct_property.write(asset, false)?;
            }

            if asset.get_object_version() >= ObjectVersion::VER_UE4_INNER_ARRAY_TAG_INFO {
                let full_len = asset.position() as i32 - length_loc;
                let new_loc = asset.position() as i32;
                asset.seek(SeekFrom::Start(length_loc as u64))?;
                let length = full_len
                    - 32
                    - match include_header {
                        true => 1,
                        false => 0,
                    };

                asset.write_i32::<LittleEndian>(length)?;
                asset.seek(SeekFrom::Start(new_loc as u64))?;
            }
        } else {
            for entry in &self.value {
                entry.write(asset, false)?;
            }
        }
        Ok((asset.position() - begin) as usize)
    }
}

impl PropertyTrait for ArrayProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        self.write_full(asset, include_header, true)
    }
}
