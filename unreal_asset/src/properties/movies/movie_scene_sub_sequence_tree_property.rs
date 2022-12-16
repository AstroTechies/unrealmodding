use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, Guid},
};

use super::{
    enums::ESectionEvaluationFlags, movie_scene_evaluation::TMovieSceneEvaluationTree,
    MovieSceneSequenceId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSubSequenceTreeEntry {
    pub sequence_id: MovieSceneSequenceId,
    pub flags: ESectionEvaluationFlags,
}

impl MovieSceneSubSequenceTreeEntry {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let sequence_id = MovieSceneSequenceId::new(asset)?;
        let flags: ESectionEvaluationFlags = ESectionEvaluationFlags::try_from(asset.read_u8()?)?;

        Ok(MovieSceneSubSequenceTreeEntry { sequence_id, flags })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.sequence_id.write(asset)?;
        asset.write_u8(self.flags as u8)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSubSequenceTree {
    pub data: TMovieSceneEvaluationTree<MovieSceneSubSequenceTreeEntry>,
}

impl MovieSceneSubSequenceTree {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let data = TMovieSceneEvaluationTree::read(asset, |reader| {
            MovieSceneSubSequenceTreeEntry::new(reader)
        })?;

        Ok(MovieSceneSubSequenceTree { data })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.data.write(asset, |writer, node| {
            node.write(writer)?;
            Ok(())
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSubSequenceTreeProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneSubSequenceTree,
}
impl_property_data_trait!(MovieSceneSubSequenceTreeProperty);

impl MovieSceneSubSequenceTreeProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneSubSequenceTree::new(asset)?;

        Ok(MovieSceneSubSequenceTreeProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneSubSequenceTreeProperty {
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
