use std::mem::size_of;

use byteorder::LittleEndian;

use crate::error::{Error, PropertyError};
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::reader::asset_reader::AssetReader;
use crate::reader::asset_writer::AssetWriter;
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    {
        custom_version::{CustomVersion, FEditorObjectVersion},
        enums::TextHistoryType,
        ue4version::{VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT, VER_UE4_FTEXT_HISTORY},
        unreal_types::{FName, Guid},
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StrProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Option<String>,
}
impl_property_data_trait!(StrProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub culture_invariant_string: Option<String>,
    pub namespace: Option<String>,
    pub table_id: Option<FName>,
    pub flags: u32,
    pub history_type: Option<i8>,
    pub value: Option<String>,
}
impl_property_data_trait!(TextProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NameProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: FName,
}
impl_property_data_trait!(NameProperty);

impl StrProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(StrProperty {
            name,
            property_guid,
            duplication_index,
            value: asset.read_string()?,
        })
    }
}

impl PropertyTrait for StrProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let begin = asset.position();
        asset.write_string(&self.value)?;
        Ok((asset.position() - begin) as usize)
    }
}

impl TextProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let mut culture_invariant_string = None;
        let mut namespace = None;
        let mut value = None;

        if asset.get_engine_version() < VER_UE4_FTEXT_HISTORY {
            culture_invariant_string = asset.read_string()?;
            if asset.get_engine_version() >= VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT {
                namespace = asset.read_string()?;
                value = asset.read_string()?;
            } else {
                namespace = None;
                value = asset.read_string()?;
            }
        }

        let flags = asset.read_u32::<LittleEndian>()?;
        let mut history_type = None;
        let mut table_id = None;
        if asset.get_engine_version() >= VER_UE4_FTEXT_HISTORY {
            history_type = Some(asset.read_i8()?);
            let history_type: TextHistoryType = history_type.unwrap().try_into()?;

            match history_type {
                TextHistoryType::None => {
                    value = None;
                    let version: CustomVersion = asset.get_custom_version::<FEditorObjectVersion>();
                    if version.version
                        >= FEditorObjectVersion::CultureInvariantTextSerializationKeyStability
                            as i32
                    {
                        let has_culture_invariant_string = asset.read_i32::<LittleEndian>()? == 1;
                        if has_culture_invariant_string {
                            culture_invariant_string = asset.read_string()?;
                        }
                    }
                }
                TextHistoryType::Base => {
                    namespace = asset.read_string()?;
                    value = asset.read_string()?;
                    culture_invariant_string = asset.read_string()?;
                }
                TextHistoryType::StringTableEntry => {
                    table_id = Some(asset.read_fname()?);
                    value = asset.read_string()?;
                }
                _ => {
                    return Err(Error::unimplemented(format!(
                        "Unimplemented reader for {:?}",
                        history_type
                    )));
                }
            }
        }

        Ok(TextProperty {
            name,
            property_guid,
            duplication_index,
            culture_invariant_string,
            namespace,
            table_id,
            flags,
            history_type,
            value,
        })
    }
}

impl PropertyTrait for TextProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let begin = asset.position();

        if asset.get_engine_version() < VER_UE4_FTEXT_HISTORY {
            asset.write_string(&self.culture_invariant_string)?;
            if asset.get_engine_version() >= VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT {
                asset.write_string(&self.namespace)?;
                asset.write_string(&self.value)?;
            } else {
                asset.write_string(&self.value)?;
            }
        }
        asset.write_u32::<LittleEndian>(self.flags)?;

        if asset.get_engine_version() >= VER_UE4_FTEXT_HISTORY {
            let history_type = self
                .history_type
                .ok_or_else(|| PropertyError::property_field_none("history_type", "i8"))?;
            asset.write_i8(history_type)?;
            let history_type = history_type.try_into()?;
            match history_type {
                TextHistoryType::None => {
                    if asset.get_custom_version::<FEditorObjectVersion>().version
                        >= FEditorObjectVersion::CultureInvariantTextSerializationKeyStability
                            as i32
                    {
                        let is_empty = match &self.culture_invariant_string {
                            Some(e) => e.is_empty(),
                            None => true,
                        };
                        match is_empty {
                            true => asset.write_i32::<LittleEndian>(0)?,
                            false => {
                                asset.write_i32::<LittleEndian>(1)?;
                                asset.write_string(&self.culture_invariant_string)?;
                            }
                        }
                    }
                    Ok(())
                }
                TextHistoryType::Base => {
                    asset.write_string(&self.namespace)?;
                    asset.write_string(&self.value)?;
                    asset.write_string(&self.culture_invariant_string)?;
                    Ok(())
                }
                TextHistoryType::StringTableEntry => {
                    asset.write_fname(self.table_id.as_ref().ok_or_else(|| {
                        PropertyError::property_field_none("table_id", "FName")
                    })?)?;
                    asset.write_string(&self.value)?;
                    Ok(())
                }
                _ => Err(Error::unimplemented(format!(
                    "Unimplemented writer for {}",
                    history_type as i8
                ))),
            }?;
        }
        Ok((asset.position() - begin) as usize)
    }
}

impl NameProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_fname()?;
        Ok(NameProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for NameProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_fname(&self.value)?;
        Ok(size_of::<i32>() * 2)
    }
}
