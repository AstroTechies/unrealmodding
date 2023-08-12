//! Movie scene sub sequence tree property

use crate::properties::property_prelude::*;

use super::{
    enums::ESectionEvaluationFlags, movie_scene_evaluation::TMovieSceneEvaluationTree,
    movie_scene_sequence_id_property::MovieSceneSequenceId,
};

/// Movie scene sub sequence tree entry
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSubSequenceTreeEntry {
    /// Sequence id
    pub sequence_id: MovieSceneSequenceId,
    /// Section flags
    pub flags: ESectionEvaluationFlags,
}

impl MovieSceneSubSequenceTreeEntry {
    /// Read a `MovieSceneSubSequenceTreeEntry` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let sequence_id = MovieSceneSequenceId::new(asset)?;
        let flags: ESectionEvaluationFlags = ESectionEvaluationFlags::try_from(asset.read_u8()?)?;

        Ok(MovieSceneSubSequenceTreeEntry { sequence_id, flags })
    }

    /// Write a `MovieSceneSubSequenceTreeEntry` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.sequence_id.write(asset)?;
        asset.write_u8(self.flags as u8)?;

        Ok(())
    }
}

/// Movie scene sub sequence tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSubSequenceTree {
    /// Tree data
    pub data: TMovieSceneEvaluationTree<MovieSceneSubSequenceTreeEntry>,
}

impl MovieSceneSubSequenceTree {
    /// Read a `MovieSceneSubSequenceTree` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let data = TMovieSceneEvaluationTree::read(asset, |reader| {
            MovieSceneSubSequenceTreeEntry::new(reader)
        })?;

        Ok(MovieSceneSubSequenceTree { data })
    }

    /// Write a `MovieSceneSubSequenceTree` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.data.write(asset, |writer, node| {
            node.write(writer)?;
            Ok(())
        })?;

        Ok(())
    }
}

/// Movie scene sub sequence tree property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSubSequenceTreeProperty {
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
    pub value: MovieSceneSubSequenceTree,
}
impl_property_data_trait!(MovieSceneSubSequenceTreeProperty);

impl MovieSceneSubSequenceTreeProperty {
    /// Read a `MovieSceneSubSequenceTreeProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneSubSequenceTree::new(asset)?;

        Ok(MovieSceneSubSequenceTreeProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneSubSequenceTreeProperty {
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
