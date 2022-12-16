use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, Guid},
};

use super::movie_scene_evaluation::MovieSceneEvaluationFieldEntityTree;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationFieldEntityTreeProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneEvaluationFieldEntityTree,
}
impl_property_data_trait!(MovieSceneEvaluationFieldEntityTreeProperty);

impl MovieSceneEvaluationFieldEntityTreeProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneEvaluationFieldEntityTree::new(asset)?;

        Ok(MovieSceneEvaluationFieldEntityTreeProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneEvaluationFieldEntityTreeProperty {
    fn write<Writer: AssetWriter>(
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
