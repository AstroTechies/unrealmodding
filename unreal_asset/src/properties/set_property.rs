//! Set property

use crate::error::{Error, PropertyError};
use crate::impl_property_data_trait;
use crate::properties::{array_property::ArrayProperty, PropertyTrait};
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{FName, Guid, ToSerializedName};
use crate::unversioned::ancestry::Ancestry;

/// Set property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct SetProperty {
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
    /// Set values
    pub value: ArrayProperty,
    /// Values to be removed from the set when the engine loads this property
    pub removed_items: ArrayProperty,
}
impl_property_data_trait!(SetProperty);

impl SetProperty {
    /// Read a `SetProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let (array_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None),
        };

        let removed_items = ArrayProperty::new_no_header(
            asset,
            name.clone(),
            ancestry.with_parent(name.clone()),
            false,
            length,
            0,
            false,
            array_type.clone(),
            property_guid,
        )?;

        let items = ArrayProperty::new_no_header(
            asset,
            name.clone(),
            ancestry.clone(),
            false,
            length,
            0,
            false,
            array_type.clone(),
            property_guid,
        )?;

        Ok(SetProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            array_type,
            value: items,
            removed_items,
        })
    }
}

impl PropertyTrait for SetProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        let array_type = match !self.value.value.is_empty() {
            true => {
                let value = self.value.value[0].to_serialized_name();
                Some(asset.get_name_map().get_mut().add_fname(&value))
            }
            false => self.array_type.clone(),
        };

        if include_header {
            asset.write_fname(array_type.as_ref().ok_or_else(PropertyError::headerless)?)?;
            asset.write_property_guid(&self.property_guid)?;
        }

        let removed_items_len = self.removed_items.write_full(asset, false, false)?;
        let items_len = self.value.write_full(asset, false, false)?;
        Ok(removed_items_len + items_len)
    }
}
