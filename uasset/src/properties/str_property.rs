use std::io::Cursor;
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::error::{Error, PropertyError};
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    {
        cursor_ext::CursorExt,
        custom_version::{CustomVersion, FEditorObjectVersion},
        enums::TextHistoryType,
        ue4version::{VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT, VER_UE4_FTEXT_HISTORY},
        unreal_types::{FName, Guid},
        Asset,
    },
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct StrProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Option<String>,
}
impl_property_data_trait!(StrProperty);

#[derive(Debug, Hash, PartialEq, Eq)]
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

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct NameProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: FName,
}
impl_property_data_trait!(NameProperty);

impl StrProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(StrProperty {
            name,
            property_guid,
            duplication_index,
            value: asset.cursor.read_string()?,
        })
    }
}

impl PropertyTrait for StrProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        let begin = cursor.position();
        cursor.write_string(&self.value)?;
        Ok((cursor.position() - begin) as usize)
    }
}

impl TextProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let mut culture_invariant_string = None;
        let mut namespace = None;
        let mut value = None;

        if asset.engine_version < VER_UE4_FTEXT_HISTORY {
            culture_invariant_string = asset.cursor.read_string()?;
            if asset.engine_version >= VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT {
                namespace = asset.cursor.read_string()?;
                value = asset.cursor.read_string()?;
            } else {
                namespace = None;
                value = asset.cursor.read_string()?;
            }
        }

        let flags = asset.cursor.read_u32::<LittleEndian>()?;
        let mut history_type = None;
        let mut table_id = None;
        if asset.engine_version >= VER_UE4_FTEXT_HISTORY {
            history_type = Some(asset.cursor.read_i8()?);
            let history_type: TextHistoryType = history_type.unwrap().try_into()?;

            match history_type {
                TextHistoryType::None => {
                    value = None;
                    let version: CustomVersion = asset.get_custom_version::<FEditorObjectVersion>();
                    if version.version
                        >= FEditorObjectVersion::CultureInvariantTextSerializationKeyStability
                            as i32
                    {
                        let has_culture_invariant_string =
                            asset.cursor.read_i32::<LittleEndian>()? == 1;
                        if has_culture_invariant_string {
                            culture_invariant_string = asset.cursor.read_string()?;
                        }
                    }
                }
                TextHistoryType::Base => {
                    namespace = asset.cursor.read_string()?;
                    value = asset.cursor.read_string()?;
                    culture_invariant_string = asset.cursor.read_string()?;
                }
                TextHistoryType::StringTableEntry => {
                    table_id = Some(asset.read_fname()?);
                    value = asset.cursor.read_string()?;
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
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        let begin = cursor.position();

        if asset.engine_version < VER_UE4_FTEXT_HISTORY {
            cursor.write_string(&self.culture_invariant_string)?;
            if asset.engine_version >= VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT {
                cursor.write_string(&self.namespace)?;
                cursor.write_string(&self.value)?;
            } else {
                cursor.write_string(&self.value)?;
            }
        }
        cursor.write_u32::<LittleEndian>(self.flags)?;

        if asset.engine_version >= VER_UE4_FTEXT_HISTORY {
            let history_type = self
                .history_type
                .ok_or(PropertyError::property_field_none("history_type", "i8"))?;
            cursor.write_i8(history_type)?;
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
                            true => cursor.write_i32::<LittleEndian>(0)?,
                            false => {
                                cursor.write_i32::<LittleEndian>(1)?;
                                cursor.write_string(&self.culture_invariant_string)?;
                            }
                        }
                    }
                    Ok(())
                }
                TextHistoryType::Base => {
                    cursor.write_string(&self.namespace)?;
                    cursor.write_string(&self.value)?;
                    cursor.write_string(&self.culture_invariant_string)?;
                    Ok(())
                }
                TextHistoryType::StringTableEntry => {
                    asset.write_fname(
                        cursor,
                        self.table_id
                            .as_ref()
                            .ok_or(PropertyError::property_field_none("table_id", "FName"))?,
                    )?;
                    cursor.write_string(&self.value)?;
                    Ok(())
                }
                _ => Err(Error::unimplemented(format!(
                    "Unimplemented writer for {}",
                    history_type as i8
                ))),
            }?;
        }
        Ok((cursor.position() - begin) as usize)
    }
}

impl NameProperty {
    pub fn new(
        asset: &mut Asset,
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
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        asset.write_fname(cursor, &self.value)?;
        Ok(size_of::<i32>() * 2)
    }
}
