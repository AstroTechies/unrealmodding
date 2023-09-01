//! Array property

use unreal_asset_base::types::PackageIndexTrait;

use crate::property_prelude::*;

/// Array property
#[derive(FNameContainer, Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ArrayProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Array type
    pub array_type: Option<FName>,
    /// Array values
    pub value: Vec<Property>,
    /// Dummy property
    pub dummy_property: Option<StructProperty>,
}
impl_property_data_trait!(ArrayProperty);

impl ArrayProperty {
    /// Read an `ArrayProperty` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
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
            ancestry,
            include_header,
            length,
            duplication_index,
            serialize_struct_differently,
            array_type,
            property_guid,
        )
    }

    /// Create an `ArrayProperty` from an array of properties
    pub fn from_arr(
        name: FName,
        ancestry: Ancestry,
        array_type: Option<FName>,
        value: Vec<Property>,
    ) -> Self {
        ArrayProperty {
            name,
            ancestry,
            property_guid: None,
            array_type,
            value,
            duplication_index: 0,
            dummy_property: None,
        }
    }

    /// Read an `ArrayProperty` from an asset without reading the property header
    #[allow(clippy::too_many_arguments)]
    pub fn new_no_header<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        _include_header: bool,
        length: i64,
        duplication_index: i32,
        serialize_struct_differently: bool,
        mut array_type: Option<FName>,
        property_guid: Option<Guid>,
    ) -> Result<Self, Error> {
        let num_entries = asset.read_i32::<LE>()?;
        let mut entries = Vec::new();
        let mut name = name;

        let mut struct_length = 1;
        let mut struct_guid = None;

        let mut dummy_struct = None;

        let mut array_struct_type = None;
        if array_type.is_none() {
            if let Some(struct_data) = asset
                .get_mappings()
                .and_then(|e| e.get_property(&name, &ancestry))
                .and_then(|e| cast!(UsmapPropertyData, UsmapArrayPropertyData, &e.property_data))
            {
                array_type = Some(FName::new_dummy(
                    struct_data.inner_type.get_property_type().to_string(),
                    0,
                ));
                if let Some(inner_struct_data) = cast!(
                    UsmapPropertyData,
                    UsmapStructPropertyData,
                    struct_data.inner_type.as_ref()
                ) {
                    array_struct_type =
                        Some(FName::new_dummy(inner_struct_data.struct_type.clone(), 0));
                }
            }
        }

        if asset.has_unversioned_properties() && array_type.is_none() {
            return name.get_content(|name| Err(PropertyError::no_type(name, &ancestry).into()));
        }

        let new_ancestry = ancestry.with_parent(name.clone());

        if array_type.as_ref().is_some_and(|ty| ty == "StructProperty")
            && serialize_struct_differently
            && !asset.has_unversioned_properties()
        {
            let mut full_type = FName::from_slice("Generic");
            if asset.get_object_version() >= ObjectVersion::VER_UE4_INNER_ARRAY_TAG_INFO {
                name = asset.read_fname()?;
                if name == "None" {
                    return Ok(ArrayProperty::default());
                }

                let this_array_type = asset.read_fname()?;
                if this_array_type == "None" {
                    return Ok(ArrayProperty::default());
                }

                this_array_type.get_content(|this_array_type| {
                    array_type.as_ref().unwrap().get_content(|array_type| {
                        if this_array_type != array_type {
                            return Err(Error::invalid_file(format!(
                                "Invalid array type {} vs {}",
                                this_array_type, array_type
                            )));
                        }
                        Ok(())
                    })
                })?;

                struct_length = asset.read_i64::<LE>()?;
                full_type = asset.read_fname()?;

                struct_guid = Some(asset.read_guid()?);
                asset.read_property_guid()?;
            } else if let Some(type_override) = name
                .get_content(|name| asset.get_array_struct_type_override().get_by_key(name))
                .cloned()
            {
                full_type = asset.add_fname(&type_override);
            }

            if num_entries == 0 {
                dummy_struct = Some(StructProperty::dummy(
                    name.clone(),
                    ancestry.with_parent(name.clone()),
                    full_type.clone(),
                    struct_guid,
                ));
            }
            for _i in 0..num_entries {
                let data = StructProperty::custom_header(
                    asset,
                    name.clone(),
                    new_ancestry.clone(),
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
                let entry: Property = if array_type == "StructProperty" {
                    let struct_type = match array_struct_type {
                        Some(ref e) => Some(e.clone()),
                        None => Some(FName::from_slice("Generic")),
                    };
                    StructProperty::custom_header(
                        asset,
                        FName::new_dummy(i.to_string(), i32::MIN),
                        new_ancestry.clone(),
                        size_est_1,
                        0,
                        struct_type,
                        None,
                        None,
                    )?
                    .into()
                } else {
                    Property::from_type(
                        asset,
                        array_type,
                        FName::new_dummy(i.to_string(), i32::MIN),
                        ancestry.clone(),
                        false,
                        size_est_1,
                        size_est_2,
                        0,
                        false,
                    )?
                };

                entries.push(entry);
            }
        }

        Ok(ArrayProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            array_type,
            dummy_property: dummy_struct,
            value: entries,
        })
    }

    /// Write an `ArrayProperty` to an asset
    pub fn write_full<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
        include_header: bool,
        serialize_structs_differently: bool,
    ) -> Result<usize, Error> {
        let array_type = match !self.value.is_empty() {
            true => {
                let value = self.value[0].to_serialized_name();
                Some(asset.get_name_map().get_mut().add_fname(&value))
            }
            false => self.array_type.clone(),
        };

        if include_header {
            asset.write_fname(array_type.as_ref().ok_or_else(PropertyError::headerless)?)?;
            asset.write_property_guid(self.property_guid.as_ref())?;
        }

        let begin = asset.position();
        asset.write_i32::<LE>(self.value.len() as i32)?;

        if array_type.as_ref().is_some_and(|ty| ty == "StructProperty")
            && serialize_structs_differently
        {
            let property: &StructProperty = match !self.value.is_empty() {
                true => match &self.value[0] {
                    Property::StructProperty(ref e) => Ok(e),
                    _ => Err(PropertyError::invalid_array(format!(
                        "expected StructProperty got {}",
                        self.value[0].to_serialized_name()
                    ))),
                },
                false => match self.dummy_property {
                    Some(ref e) => Ok(e),
                    None => Err(PropertyError::invalid_array(
                        "Empty array with no dummy struct. Cannot serialize".to_string(),
                    )),
                },
            }?;

            let mut length_loc = None;
            if asset.get_object_version() >= ObjectVersion::VER_UE4_INNER_ARRAY_TAG_INFO {
                asset.write_fname(&property.name)?;
                asset.write_fname(&asset.get_name_map().get_mut().add_fname("StructProperty"))?;
                length_loc = Some(asset.position());
                asset.write_i64::<LE>(0)?;
                asset.write_fname(
                    property.struct_type.as_ref().ok_or_else(|| {
                        PropertyError::property_field_none("struct_type", "FName")
                    })?,
                )?;
                if asset.get_object_version() >= ObjectVersion::VER_UE4_STRUCT_GUID_IN_PROPERTY_TAG
                {
                    asset.write_guid(&property.property_guid.unwrap_or_default())?;
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
                        property.to_serialized_name()
                    ))),
                }?;
                struct_property.write(asset, false)?;
            }

            if asset.get_object_version() >= ObjectVersion::VER_UE4_INNER_ARRAY_TAG_INFO {
                let length_loc = length_loc.expect("Corrupted memory");
                let full_len = asset.position() - length_loc;
                let new_loc = asset.position();
                asset.seek(SeekFrom::Start(length_loc))?;
                let length = full_len
                    - 32
                    - match include_header {
                        true => 1,
                        false => 0,
                    };

                asset.write_i32::<LE>(length as i32)?;
                asset.seek(SeekFrom::Start(new_loc))?;
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
    fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        self.write_full(asset, include_header, true)
    }
}
