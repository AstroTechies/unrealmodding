//! String properties

use std::mem::size_of;

use byteorder::LittleEndian;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use unreal_asset_proc_macro::FNameContainer;

use crate::custom_version::{CustomVersion, FEditorObjectVersion};
use crate::error::{Error, PropertyError};
use crate::impl_property_data_trait;
use crate::object_version::ObjectVersion;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{fname::FName, Guid};
use crate::unversioned::ancestry::Ancestry;

/// Text history type
#[derive(
    FNameContainer, Debug, Hash, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive,
)]
#[repr(i8)]
pub enum TextHistoryType {
    /// None
    None = -1,
    /// Base
    Base = 0,
    /// Named format
    NamedFormat,
    /// Ordered format
    OrderedFormat,
    /// Argument format
    ArgumentFormat,
    /// As number
    AsNumber,
    /// As percentage
    AsPercent,
    /// As currency
    AsCurrency,
    /// As date
    AsDate,
    /// As time
    AsTime,
    /// As datetime
    AsDateTime,
    /// Transform
    Transform,
    /// String table entry
    StringTableEntry,
    /// Text generator
    TextGenerator,
    /// Uncertain, Back 4 Blood specific serialization
    RawText,
}

impl Default for TextHistoryType {
    fn default() -> Self {
        Self::None
    }
}

/// String property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct StrProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// FString value
    pub value: Option<String>,
}
impl_property_data_trait!(StrProperty);

/// Text property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Culture invariant string
    pub culture_invariant_string: Option<String>,
    /// Namespace
    pub namespace: Option<String>,
    /// String table id
    pub table_id: Option<FName>,
    /// Flags
    pub flags: u32,
    /// History type
    pub history_type: TextHistoryType,
    /// FString value
    pub value: Option<String>,
}
impl_property_data_trait!(TextProperty);

/// Name property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct NameProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// FName value
    pub value: FName,
}
impl_property_data_trait!(NameProperty);

impl StrProperty {
    /// Read a `StrProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        Ok(StrProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value: asset.read_fstring()?,
        })
    }
}

impl PropertyTrait for StrProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let begin = asset.position();
        asset.write_fstring(self.value.as_deref())?;
        Ok((asset.position() - begin) as usize)
    }
}

impl TextProperty {
    /// Read a `TextProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let mut culture_invariant_string = None;
        let mut namespace = None;
        let mut value = None;

        if asset.get_object_version() < ObjectVersion::VER_UE4_FTEXT_HISTORY {
            culture_invariant_string = asset.read_fstring()?;
            if asset.get_object_version()
                >= ObjectVersion::VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT
            {
                namespace = asset.read_fstring()?;
                value = asset.read_fstring()?;
            } else {
                namespace = None;
                value = asset.read_fstring()?;
            }
        }

        let flags = asset.read_u32::<LittleEndian>()?;
        let mut history_type = TextHistoryType::Base;
        let mut table_id = None;
        if asset.get_object_version() >= ObjectVersion::VER_UE4_FTEXT_HISTORY {
            history_type = TextHistoryType::try_from(asset.read_i8()?)?;

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
                            culture_invariant_string = asset.read_fstring()?;
                        }
                    }
                }
                TextHistoryType::Base => {
                    namespace = asset.read_fstring()?;
                    value = asset.read_fstring()?;
                    culture_invariant_string = asset.read_fstring()?;
                }
                TextHistoryType::StringTableEntry => {
                    table_id = Some(asset.read_fname()?);
                    value = asset.read_fstring()?;
                }
                _ => {
                    return Err(Error::unimplemented(format!(
                        "Unimplemented reader for {history_type:?}"
                    )));
                }
            }
        }

        Ok(TextProperty {
            name,
            ancestry,
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
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        let begin = asset.position();

        if asset.get_object_version() < ObjectVersion::VER_UE4_FTEXT_HISTORY {
            asset.write_fstring(self.culture_invariant_string.as_deref())?;
            if asset.get_object_version()
                >= ObjectVersion::VER_UE4_ADDED_NAMESPACE_AND_KEY_DATA_TO_FTEXT
            {
                asset.write_fstring(self.namespace.as_deref())?;
                asset.write_fstring(self.value.as_deref())?;
            } else {
                asset.write_fstring(self.value.as_deref())?;
            }
        }
        asset.write_u32::<LittleEndian>(self.flags)?;

        if asset.get_object_version() >= ObjectVersion::VER_UE4_FTEXT_HISTORY {
            let history_type = self.history_type;
            asset.write_i8(history_type.into())?;
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
                                asset.write_fstring(self.culture_invariant_string.as_deref())?;
                            }
                        }
                    }
                    Ok(())
                }
                TextHistoryType::Base => {
                    asset.write_fstring(self.namespace.as_deref())?;
                    asset.write_fstring(self.value.as_deref())?;
                    asset.write_fstring(self.culture_invariant_string.as_deref())?;
                    Ok(())
                }
                TextHistoryType::StringTableEntry => {
                    asset.write_fname(self.table_id.as_ref().ok_or_else(|| {
                        PropertyError::property_field_none("table_id", "FName")
                    })?)?;
                    asset.write_fstring(self.value.as_deref())?;
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
    /// Read a `NameProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_fname()?;
        Ok(NameProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for NameProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_fname(&self.value)?;
        Ok(size_of::<i32>() * 2)
    }
}
