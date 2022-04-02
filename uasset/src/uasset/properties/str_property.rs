use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::TryFromPrimitiveError;

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, ue4version::{VER_UE4_FTEXT_HISTORY, VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT}, flags::TextHistoryType, Asset, custom_version::{FEditorObjectVersion, CustomVersion}}, optional_guid};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct StrProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: String
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TextProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub culture_invariant_string: Option<String>,
    pub namespace: Option<String>,
    pub table_id: Option<FName>,
    pub flags: u32,
    pub history_type: Option<i8>,
    pub value: Option<String>
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct NameProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: FName
}


impl StrProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        Ok(StrProperty {
            name,
            property_guid,
            value: cursor.read_string()?
        })
    }
}

impl TextProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, engine_version: i32, asset: &mut Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        let mut culture_invariant_string = None;
        let mut namespace = None;
        let mut value = None;

        if engine_version < VER_UE4_FTEXT_HISTORY {
            culture_invariant_string = Some(cursor.read_string()?);
            if engine_version >= VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT {
                namespace = Some(cursor.read_string()?);
                value = Some(cursor.read_string()?);
            } else {
                namespace = None;
                value = Some(cursor.read_string()?);
            }
        }

        let flags = cursor.read_u32::<LittleEndian>()?;
        let mut history_type = None;
        let mut table_id = None;
        if engine_version >= VER_UE4_FTEXT_HISTORY {
            history_type = Some(cursor.read_i8()?);
            let history_type: TextHistoryType = history_type.unwrap().try_into().map_err(|e: TryFromPrimitiveError<TextHistoryType>| Error::new(ErrorKind::Other, e.to_string()))?;

            match history_type {
                TextHistoryType::None => {
                    value = None;
                    let version: CustomVersion = asset.get_custom_version("FEditorObjectVersion").ok_or(Error::new(ErrorKind::Other, "Unknown custom version"))?;
                    if version.version >= FEditorObjectVersion::CultureInvariantTextSerializationKeyStability as i32 {
                        let has_culture_invariant_string = cursor.read_i32::<LittleEndian>()? == 1;
                        if has_culture_invariant_string {
                            culture_invariant_string = Some(cursor.read_string()?);
                        }
                    }
                }
                TextHistoryType::Base => {
                    namespace = Some(cursor.read_string()?);
                    value = Some(cursor.read_string()?);
                    culture_invariant_string = Some(cursor.read_string()?);
                }
                TextHistoryType::StringTableEntry => {
                    table_id = Some(asset.read_fname()?);
                    value = Some(cursor.read_string()?);
                }
                _ => {
                    return Err(Error::new(ErrorKind::Other, format!("Unimplemented reader for {:?}", history_type)));
                }
            }
        }

        Ok(TextProperty {
            name,
            property_guid, culture_invariant_string, namespace, table_id, flags, history_type, value
        })
    }
}

impl NameProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &mut Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let value = asset.read_fname()?;
        Ok(NameProperty {
            name,
            property_guid,
            value
        })
    }
}
