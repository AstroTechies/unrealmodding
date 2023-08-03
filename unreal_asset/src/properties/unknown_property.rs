//! Unknown property

use unreal_asset_proc_macro::FNameContainer;
use unreal_helpers::Guid;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::fname::FName;
use crate::unversioned::ancestry::Ancestry;

/// Unknown property
///
/// This gets created when an unknown property was encountered while deserializing
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct UnknownProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Raw data
    pub value: Vec<u8>,
    /// Serialized type
    pub serialized_type: FName,
}
impl_property_data_trait!(UnknownProperty);

impl UnknownProperty {
    /// Read an `UnknownProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        length: i64,
        duplication_index: i32,
        serialized_type: FName,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let mut value = vec![0u8; length as usize];
        asset.read_exact(&mut value)?;

        Ok(UnknownProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
            serialized_type,
        })
    }
}

impl PropertyTrait for UnknownProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_all(&self.value)?;
        Ok(self.value.len())
    }
}
