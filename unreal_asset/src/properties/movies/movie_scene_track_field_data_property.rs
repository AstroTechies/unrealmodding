use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, Guid},
};

use super::{movie_scene_evaluation::TMovieSceneEvaluationTree, MovieSceneTrackIdentifier};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneTrackFieldData {
    pub field: TMovieSceneEvaluationTree<MovieSceneTrackIdentifier>,
}

impl MovieSceneTrackFieldData {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let field = TMovieSceneEvaluationTree::read(asset, |reader| {
            MovieSceneTrackIdentifier::new(reader)
        })?;

        Ok(MovieSceneTrackFieldData { field })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.field.write(asset, |writer, node| {
            node.write(writer)?;
            Ok(())
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneTrackFieldDataProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneTrackFieldData,
}
impl_property_data_trait!(MovieSceneTrackFieldDataProperty);

impl MovieSceneTrackFieldDataProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid(asset, include_header);

        let value = MovieSceneTrackFieldData::new(asset)?;

        Ok(MovieSceneTrackFieldDataProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneTrackFieldDataProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, crate::error::Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        self.value.write(asset)?;

        Ok((asset.position() - begin) as usize)
    }
}
