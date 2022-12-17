use std::{fmt::Debug, hash::Hash};

use byteorder::LittleEndian;

use crate::{
    error::Error,
    properties::core_uobject::FFrameNumberRange,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FEntry {
    pub start_index: i32,
    pub size: i32,
    pub capacity: i32,
}

impl FEntry {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let start_index = asset.read_i32::<LittleEndian>()?;
        let size = asset.read_i32::<LittleEndian>()?;
        let capacity = asset.read_i32::<LittleEndian>()?;

        Ok(FEntry {
            start_index,
            size,
            capacity,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.start_index)?;
        asset.write_i32::<LittleEndian>(self.size)?;
        asset.write_i32::<LittleEndian>(self.capacity)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EvaluationTreeEntryHandle {
    pub entry_index: i32,
}

impl EvaluationTreeEntryHandle {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let entry_index = asset.read_i32::<LittleEndian>()?;

        Ok(EvaluationTreeEntryHandle { entry_index })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.entry_index)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationTreeNodeHandle {
    pub children_handle: EvaluationTreeEntryHandle,
    pub index: i32,
}

impl MovieSceneEvaluationTreeNodeHandle {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let children_handle = EvaluationTreeEntryHandle::new(asset)?;
        let index = asset.read_i32::<LittleEndian>()?;

        Ok(MovieSceneEvaluationTreeNodeHandle {
            children_handle,
            index,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.children_handle.write(asset)?;
        asset.write_i32::<LittleEndian>(self.index)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TEvaluationTreeEntryContainer<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    pub entries: Vec<FEntry>,
    pub items: Vec<T>,
}

impl<T> TEvaluationTreeEntryContainer<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    pub fn new(entries: Vec<FEntry>, items: Vec<T>) -> Self {
        TEvaluationTreeEntryContainer { entries, items }
    }

    pub fn read<Reader: AssetReader>(
        asset: &mut Reader,
        item_reader: fn(&mut Reader) -> Result<T, Error>,
    ) -> Result<Self, Error> {
        let entries_amount = asset.read_i32::<LittleEndian>()?;
        let mut entries = Vec::with_capacity(entries_amount as usize);

        for _ in 0..entries_amount {
            entries.push(FEntry::new(asset)?);
        }

        let items_amount = asset.read_i32::<LittleEndian>()?;
        let mut items = Vec::with_capacity(items_amount as usize);

        for _ in 0..entries_amount {
            items.push(item_reader(asset)?);
        }

        Ok(TEvaluationTreeEntryContainer { entries, items })
    }

    pub fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        node_writer: fn(&mut Writer, &T) -> Result<(), Error>,
    ) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.entries.len() as i32)?;
        for entry in &self.entries {
            entry.write(asset)?;
        }

        asset.write_i32::<LittleEndian>(self.items.len() as i32)?;
        for item in &self.items {
            node_writer(asset, item)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TMovieSceneEvaluationTree<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    pub root_node: MovieSceneEvaluationTreeNode,
    pub child_nodes: TEvaluationTreeEntryContainer<MovieSceneEvaluationTreeNode>,
    pub data: TEvaluationTreeEntryContainer<T>,
}

impl<T> TMovieSceneEvaluationTree<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    pub fn new(
        root_node: MovieSceneEvaluationTreeNode,
        child_nodes: TEvaluationTreeEntryContainer<MovieSceneEvaluationTreeNode>,
        data: TEvaluationTreeEntryContainer<T>,
    ) -> Self {
        TMovieSceneEvaluationTree {
            root_node,
            child_nodes,
            data,
        }
    }

    pub fn read<Reader: AssetReader>(
        asset: &mut Reader,
        data_node_reader: fn(&mut Reader) -> Result<T, Error>,
    ) -> Result<Self, Error> {
        let root_node = MovieSceneEvaluationTreeNode::new(asset)?;
        let child_nodes = TEvaluationTreeEntryContainer::read(asset, |reader| {
            MovieSceneEvaluationTreeNode::new(reader)
        })?;

        let data = TEvaluationTreeEntryContainer::read(asset, data_node_reader)?;

        Ok(TMovieSceneEvaluationTree {
            root_node,
            child_nodes,
            data,
        })
    }

    pub fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        data_node_writer: fn(&mut Writer, &T) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.root_node.write(asset)?;

        self.child_nodes.write(asset, |writer, node| {
            node.write(writer)?;
            Ok(())
        })?;

        self.data.write(asset, data_node_writer)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationTreeNode {
    pub range: FFrameNumberRange,
    pub parent: MovieSceneEvaluationTreeNodeHandle,
    pub children_id: EvaluationTreeEntryHandle,
    pub data_id: EvaluationTreeEntryHandle,
}

impl MovieSceneEvaluationTreeNode {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let range = FFrameNumberRange::new(asset)?;
        let parent = MovieSceneEvaluationTreeNodeHandle::new(asset)?;
        let children_id = EvaluationTreeEntryHandle::new(asset)?;
        let data_id = EvaluationTreeEntryHandle::new(asset)?;

        Ok(MovieSceneEvaluationTreeNode {
            range,
            parent,
            children_id,
            data_id,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.range.write(asset)?;
        self.parent.write(asset)?;
        self.children_id.write(asset)?;
        self.data_id.write(asset)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FEntityAndMetaDataIndex {
    pub entity_index: i32,
    pub meta_data_index: i32,
}

impl FEntityAndMetaDataIndex {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let entity_index = asset.read_i32::<LittleEndian>()?;
        let meta_data_index = asset.read_i32::<LittleEndian>()?;

        Ok(FEntityAndMetaDataIndex {
            entity_index,
            meta_data_index,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.entity_index)?;
        asset.write_i32::<LittleEndian>(self.meta_data_index)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationFieldEntityTree {
    pub serialized_data: TMovieSceneEvaluationTree<FEntityAndMetaDataIndex>,
}

impl MovieSceneEvaluationFieldEntityTree {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let serialized_data =
            TMovieSceneEvaluationTree::read(asset, |reader| FEntityAndMetaDataIndex::new(reader))?;

        Ok(MovieSceneEvaluationFieldEntityTree { serialized_data })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.serialized_data.write(asset, |writer, node| {
            node.write(writer)?;
            Ok(())
        })?;

        Ok(())
    }
}
