//! Guid property

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

/// Guid property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct GuidProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Guid value
    pub value: Guid,
}
impl_property_data_trait!(GuidProperty);

impl GuidProperty {
    /// Read a `GuidProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_guid()?;
        Ok(GuidProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for GuidProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_guid(&self.value)?;
        Ok(16)
    }
}
