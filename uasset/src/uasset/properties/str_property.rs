use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, ue4version::{VER_UE4_FTEXT_HISTORY, VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT}, flags::TextHistoryType, Asset, custom_version::FEditorObjectVersion}, optional_guid};

#[derive(Debug)]
pub struct StrProperty {
    property_guid: Option<Guid>,
    value: String
}

#[derive(Debug)]
pub struct TextProperty {
    property_guid: Option<Guid>,
    culture_invariant_string: Option<String>,
    namespace: Option<String>,
    table_id: Option<FName>,
    flags: u32,
    history_type: Option<i8>,
    value: String
}

#[derive(Debug)]
pub struct NameProperty {
    property_guid: Option<Guid>,
    value: FName
}


impl StrProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        Ok(StrProperty {
            property_guid,
            value: cursor.read_string()?
        })
    }
}

impl TextProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool, engine_version: i32, asset: &Asset) -> Result<Self, Error> {
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
            history_type = cursor.read_i8()?;
            let history_type = history_type.into()?;

            match history_type {
                TextHistoryType::None => {
                    value = None;
                    let version: FEditorObjectVersion = asset.get_custom_version("FEditorObjectVersion").into()?;
                    if version >= FEditorObjectVersion::CultureInvariantTextSerializationKeyStability {
                        let has_culture_invariant_string = cursor.read_i32::<LittleEndian>() == 1;
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
                    return Error::new(ErrorKind::Other, format!("Unimplemented reader for {}", history_type));
                }
            }
        }

        Ok(TextProperty {
            property_guid, culture_invariant_string, namespace, table_id, flags, history_type, value
        })
    }
}

impl NameProperty {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, include_header: bool, asset: &Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);
        let value = asset.read_fname()?;
        Ok(NameProperty {
            property_guid,
            value
        })
    }
}