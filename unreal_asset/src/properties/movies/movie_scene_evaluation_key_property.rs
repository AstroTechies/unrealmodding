use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, Guid},
};

use super::{
    movie_scene_sequence_id_property::MovieSceneSequenceId,
    movie_scene_track_identifier_property::MovieSceneTrackIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationKey {
    pub sequence_id: MovieSceneSequenceId,
    pub track_identifier: MovieSceneTrackIdentifier,
    pub section_index: u32,
}

impl MovieSceneEvaluationKey {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let sequence_id = MovieSceneSequenceId::new(asset)?;
        let track_identifier = MovieSceneTrackIdentifier::new(asset)?;
        let section_index = asset.read_u32::<LittleEndian>()?;

        Ok(MovieSceneEvaluationKey {
            sequence_id,
            track_identifier,
            section_index,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.sequence_id.write(asset)?;
        self.track_identifier.write(asset)?;
        asset.write_u32::<LittleEndian>(self.section_index)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationKeyProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneEvaluationKey,
}
impl_property_data_trait!(MovieSceneEvaluationKeyProperty);

impl MovieSceneEvaluationKeyProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneEvaluationKey::new(asset)?;

        Ok(MovieSceneEvaluationKeyProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneEvaluationKeyProperty {
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
