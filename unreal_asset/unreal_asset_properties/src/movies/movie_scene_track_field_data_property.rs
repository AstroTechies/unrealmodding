//! Movie scene track field data property

use crate::property_prelude::*;

use super::{
    movie_scene_evaluation::TMovieSceneEvaluationTree,
    movie_scene_track_identifier_property::MovieSceneTrackIdentifier,
};

/// Movie scene track field data
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct MovieSceneTrackFieldData {
    /// Data tree
    pub field: TMovieSceneEvaluationTree<MovieSceneTrackIdentifier>,
}

impl MovieSceneTrackFieldData {
    /// Read `MovieSceneTrackFieldData` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let field = TMovieSceneEvaluationTree::read(asset, |reader| {
            MovieSceneTrackIdentifier::new(reader)
        })?;

        Ok(MovieSceneTrackFieldData { field })
    }

    /// Write `MovieSceneTrackFieldData` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.field.write(asset, |writer, node| {
            node.write(writer)?;
            Ok(())
        })?;

        Ok(())
    }
}

/// Movie scene track field data property
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct MovieSceneTrackFieldDataProperty {
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
    pub value: MovieSceneTrackFieldData,
}
impl_property_data_trait!(MovieSceneTrackFieldDataProperty);

impl MovieSceneTrackFieldDataProperty {
    /// Read a `MovieSceneTrackFieldDataProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneTrackFieldData::new(asset)?;

        Ok(MovieSceneTrackFieldDataProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneTrackFieldDataProperty {
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
