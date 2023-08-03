//! Movie scene track identifier property

use byteorder::LE;

use unreal_asset_proc_macro::FNameContainer;
use unreal_helpers::Guid;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    types::fname::FName,
    unversioned::ancestry::Ancestry,
};

/// Movie scene track identifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneTrackIdentifier {
    /// Identifier value
    pub value: u32,
}

impl MovieSceneTrackIdentifier {
    /// Read a `MovieSceneTrackIdentifier` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let value = asset.read_u32::<LE>()?;

        Ok(MovieSceneTrackIdentifier { value })
    }

    /// Write a `MovieSceneTrackIdentifier` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_u32::<LE>(self.value)?;
        Ok(())
    }
}

/// Movie scene track identifier property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneTrackIdentifierProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Identifier
    #[container_ignore]
    pub value: MovieSceneTrackIdentifier,
}
impl_property_data_trait!(MovieSceneTrackIdentifierProperty);

impl MovieSceneTrackIdentifierProperty {
    /// Read a `MovieSceneTrackIdentifierProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneTrackIdentifier::new(asset)?;

        Ok(MovieSceneTrackIdentifierProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneTrackIdentifierProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        self.value.write(asset)?;

        Ok((asset.position() - begin) as usize)
    }
}
