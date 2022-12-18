use std::io::SeekFrom;
use std::mem::size_of;

use byteorder::LittleEndian;

use crate::custom_version::{
    FEditorObjectVersion, FFortniteMainBranchObjectVersion, FSequencerObjectVersion,
};
use crate::error::{Error, PropertyError};
use crate::object_version::ObjectVersion;
use crate::properties::{Property, PropertyTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, Guid};
use crate::{cast, impl_property_data_trait};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
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

    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        length: i64,
        _duplication_index: i32,
    ) -> Result<Self, Error> {
        let mut struct_type = None;
        let mut struct_guid = None;
        let mut property_guid = None;

        if include_header {
            struct_type = Some(asset.read_fname()?);
            if asset.get_object_version() >= ObjectVersion::VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG {
                let mut guid = [0u8; 16];
                asset.read_exact(&mut guid)?;
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

    pub fn custom_header<Reader: AssetReader>(
        asset: &mut Reader,
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
            if e.content == "FloatRange" {
                // FloatRange is a special case; it can either be manually serialized as two floats (TRange<float>) or as a regular struct (FFloatRange), but the first is overridden to use the same name as the second
                // The best solution is to just check and see if the next bit is an FName or not

                let name_map_index = asset.read_i32::<LittleEndian>()?;
                asset.seek(SeekFrom::Current(-(size_of::<u32>() as i64)))?;

                if name_map_index >= 0
                    && name_map_index < asset.get_name_map_index_list().len() as i32
                {
                    custom_serialization = asset.get_name_reference(name_map_index) != "LowerBound";
                } else {
                    custom_serialization = true;
                }
            }

            if e.content == "RichCurveKey"
                && asset.get_object_version() < ObjectVersion::VER_UE4_SERIALIZE_RICH_CURVE_KEY
            {
                custom_serialization = false;
            }

            if e.content == "MovieSceneTrackIdentifier"
                && asset.get_custom_version::<FEditorObjectVersion>().version
                    < FEditorObjectVersion::MovieSceneMetaDataSerialization as i32
            {
                custom_serialization = false;
            }

            if e.content == "MovieSceneFloatChannel"
                && asset
                    .get_custom_version::<FSequencerObjectVersion>()
                    .version
                    < FSequencerObjectVersion::SerializeFloatChannelCompletely as i32
                && asset
                    .get_custom_version::<FFortniteMainBranchObjectVersion>()
                    .version
                    < FFortniteMainBranchObjectVersion::SerializeFloatChannelShowCurve as i32
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

            Ok(StructProperty {
                name,
                struct_type,
                struct_guid,
                property_guid,
                duplication_index,
                serialize_none: true,
                value,
            })
        } else {
            let mut values = Vec::new();
            let mut property = Property::new(asset, true)?;
            while property.is_some() {
                values.push(property.unwrap());
                property = Property::new(asset, true)?;
            }

            Ok(StructProperty {
                name,
                struct_type,
                struct_guid,
                property_guid,
                duplication_index,
                serialize_none: true,
                value: values,
            })
        }
    }

    pub fn write_with_type<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
        struct_type: Option<FName>,
    ) -> Result<usize, Error> {
        if include_header {
            asset.write_fname(struct_type.as_ref().ok_or_else(PropertyError::headerless)?)?;
            if asset.get_object_version() >= ObjectVersion::VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG {
                asset.write_all(&self.struct_guid.ok_or_else(PropertyError::headerless)?)?;
            }
            asset.write_property_guid(&self.property_guid)?;
        }

        let mut has_custom_serialization = match struct_type {
            Some(ref e) => Property::has_custom_serialization(&e.content),
            None => false,
        };

        if let Some(ref struct_type) = struct_type {
            if struct_type.content == "FloatRange" {
                has_custom_serialization = self.value.len() == 1
                    && cast!(Property, FloatRangeProperty, &self.value[0]).is_some();
            }

            if struct_type.content == "RichCurveKey"
                && asset.get_object_version() < ObjectVersion::VER_UE4_SERIALIZE_RICH_CURVE_KEY
            {
                has_custom_serialization = false;
            }

            if struct_type.content == "MovieSceneTrackIdentifier"
                && asset.get_custom_version::<FEditorObjectVersion>().version
                    < FEditorObjectVersion::MovieSceneMetaDataSerialization as i32
            {
                has_custom_serialization = false;
            }

            if struct_type.content == "MovieSceneFloatChannel"
                && asset
                    .get_custom_version::<FSequencerObjectVersion>()
                    .version
                    < FSequencerObjectVersion::SerializeFloatChannelCompletely as i32
                && asset
                    .get_custom_version::<FFortniteMainBranchObjectVersion>()
                    .version
                    < FFortniteMainBranchObjectVersion::SerializeFloatChannelShowCurve as i32
            {
                has_custom_serialization = false;
            }
        }

        if has_custom_serialization {
            if self.value.len() != 1 {
                return Err(PropertyError::invalid_struct(format!(
                    "Structs with type {} must have exactly 1 entry",
                    struct_type
                        .as_ref()
                        .map(|e| e.content.to_owned())
                        .unwrap_or_else(|| "Generic".to_string())
                ))
                .into());
            }
            self.value[0].write(asset, false)
        } else if self.value.is_empty() && !self.serialize_none {
            Ok(0)
        } else {
            let begin = asset.position();
            for entry in &self.value {
                Property::write(entry, asset, true)?;
            }
            asset.write_fname(&FName::from_slice("None"))?;
            Ok((asset.position() - begin) as usize)
        }
    }
}

impl PropertyTrait for StructProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        self.write_with_type(asset, include_header, self.struct_type.clone())
    }
}
