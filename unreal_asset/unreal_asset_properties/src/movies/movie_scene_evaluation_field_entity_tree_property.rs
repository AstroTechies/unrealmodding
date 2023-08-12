//! Movie scene evaluation field entity tree property

use crate::property_prelude::*;

use super::movie_scene_evaluation::MovieSceneEvaluationFieldEntityTree;

/// Movie scene evaluation field entity tree property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationFieldEntityTreeProperty {
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
    pub value: MovieSceneEvaluationFieldEntityTree,
}
impl_property_data_trait!(MovieSceneEvaluationFieldEntityTreeProperty);

impl MovieSceneEvaluationFieldEntityTreeProperty {
    /// Read a `MovieSceneEvaluationFieldEntityTreeProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneEvaluationFieldEntityTree::new(asset)?;

        Ok(MovieSceneEvaluationFieldEntityTreeProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneEvaluationFieldEntityTreeProperty {
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
