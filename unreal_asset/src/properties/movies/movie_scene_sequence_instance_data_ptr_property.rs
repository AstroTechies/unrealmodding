//! Movie scene sequence instance data pointer property

use byteorder::LE;

use unreal_asset_proc_macro::FNameContainer;
use unreal_helpers::Guid;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    types::{fname::FName, PackageIndex},
    unversioned::ancestry::Ancestry,
};

/// Movie scene sequence instance data pointer property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSequenceInstanceDataPtrProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    #[container_ignore]
    pub value: PackageIndex,
}
impl_property_data_trait!(MovieSceneSequenceInstanceDataPtrProperty);

impl MovieSceneSequenceInstanceDataPtrProperty {
    /// Read a `MovieSceneSequenceInstanceDataPtrProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = PackageIndex::new(asset.read_i32::<LE>()?);

        Ok(MovieSceneSequenceInstanceDataPtrProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneSequenceInstanceDataPtrProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();
        asset.write_i32::<LE>(self.value.index)?;

        Ok((asset.position() - begin) as usize)
    }
}
