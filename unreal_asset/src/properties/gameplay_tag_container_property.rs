//! Gameplay tag container property

use std::mem::size_of;

use byteorder::LE;
use unreal_asset_proc_macro::FNameContainer;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{fname::FName, Guid};
use crate::unversioned::ancestry::Ancestry;

/// Gameplay tag container property
#[derive(FNameContainer, Debug, Hash, Clone, PartialEq, Eq)]
pub struct GameplayTagContainerProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Gameplay tags
    pub value: Vec<FName>,
}
impl_property_data_trait!(GameplayTagContainerProperty);

impl GameplayTagContainerProperty {
    /// Read a `GameplayTagContainerProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let length = asset.read_i32::<LE>()?;
        let mut value = Vec::with_capacity(length as usize);
        for _i in 0..length as usize {
            value.push(asset.read_fname()?);
        }

        Ok(GameplayTagContainerProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for GameplayTagContainerProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LE>(self.value.len() as i32)?;

        let mut total_size = size_of::<i32>();
        for entry in &self.value {
            asset.write_fname(entry)?;
            total_size += size_of::<i32>() * 2;
        }

        Ok(total_size)
    }
}
