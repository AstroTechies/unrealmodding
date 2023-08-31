//! Movie scene evaluation

use std::fmt::Debug;
use std::hash::Hash;

use unreal_asset_base::types::movie::FFrameNumberRange;
use unreal_asset_base::types::PackageIndexTrait;

use crate::property_prelude::*;

/// Movie scene evaluation entry
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct FEntry {
    /// Start index
    pub start_index: i32,
    /// Size
    pub size: i32,
    /// Capacity
    pub capacity: i32,
}

impl FEntry {
    /// Read an `FEntry` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let start_index = asset.read_i32::<LE>()?;
        let size = asset.read_i32::<LE>()?;
        let capacity = asset.read_i32::<LE>()?;

        Ok(FEntry {
            start_index,
            size,
            capacity,
        })
    }

    /// Write an `FEntry` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        asset.write_i32::<LE>(self.start_index)?;
        asset.write_i32::<LE>(self.size)?;
        asset.write_i32::<LE>(self.capacity)?;
        Ok(())
    }
}

/// Evaluation tree entry handle
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct EvaluationTreeEntryHandle {
    /// Entry index
    pub entry_index: i32,
}

impl EvaluationTreeEntryHandle {
    /// Read an `EvaluationTreeEntryHandle` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let entry_index = asset.read_i32::<LE>()?;

        Ok(EvaluationTreeEntryHandle { entry_index })
    }

    /// Write an `EvaluationTreeEntryHandle` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        asset.write_i32::<LE>(self.entry_index)?;
        Ok(())
    }
}

/// Movie scene evaluation tree node handle
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationTreeNodeHandle {
    /// Children handle
    pub children_handle: EvaluationTreeEntryHandle,
    /// Index
    pub index: i32,
}

impl MovieSceneEvaluationTreeNodeHandle {
    /// Read a `MovieSceneEvaluationTreeNodeHandle` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let children_handle = EvaluationTreeEntryHandle::new(asset)?;
        let index = asset.read_i32::<LE>()?;

        Ok(MovieSceneEvaluationTreeNodeHandle {
            children_handle,
            index,
        })
    }

    /// Write a `MovieSceneEvaluationTreeNodeHandle` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        self.children_handle.write(asset)?;
        asset.write_i32::<LE>(self.index)?;

        Ok(())
    }
}

/// Generic evaluation tree entry container
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TEvaluationTreeEntryContainer<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    /// Entries
    #[container_ignore]
    pub entries: Vec<FEntry>,
    /// Items
    pub items: Vec<T>,
}

impl<T> TEvaluationTreeEntryContainer<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    /// Create a new `TEvaluationTreeEntryContainer` instance
    pub fn new(entries: Vec<FEntry>, items: Vec<T>) -> Self {
        TEvaluationTreeEntryContainer { entries, items }
    }

    /// Read a `TEvaluationTreeEntryContainer` from an asset
    pub fn read<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        item_reader: fn(&mut Reader) -> Result<T, Error>,
    ) -> Result<Self, Error> {
        let entries_amount = asset.read_i32::<LE>()?;
        let mut entries = Vec::with_capacity(entries_amount as usize);

        for _ in 0..entries_amount {
            entries.push(FEntry::new(asset)?);
        }

        let items_amount = asset.read_i32::<LE>()?;
        let mut items = Vec::with_capacity(items_amount as usize);

        for _ in 0..entries_amount {
            items.push(item_reader(asset)?);
        }

        Ok(TEvaluationTreeEntryContainer { entries, items })
    }

    /// Write a `TEvaluationTreeEntryContainer` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
        node_writer: fn(&mut Writer, &T) -> Result<(), Error>,
    ) -> Result<(), Error> {
        asset.write_i32::<LE>(self.entries.len() as i32)?;
        for entry in &self.entries {
            entry.write(asset)?;
        }

        asset.write_i32::<LE>(self.items.len() as i32)?;
        for item in &self.items {
            node_writer(asset, item)?;
        }

        Ok(())
    }
}

/// Generic movie scene evaluation tree
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TMovieSceneEvaluationTree<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    /// Root node
    #[container_ignore]
    pub root_node: MovieSceneEvaluationTreeNode,
    /// Child nodes
    #[container_ignore]
    pub child_nodes: TEvaluationTreeEntryContainer<MovieSceneEvaluationTreeNode>,
    /// Data
    pub data: TEvaluationTreeEntryContainer<T>,
}

impl<T> TMovieSceneEvaluationTree<T>
where
    T: Debug + Clone + PartialEq + Eq + Hash,
{
    /// Create a new `TMovieSceneEvaluationTree` instance
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

    /// Read a `TMovieSceneEvaluationTree` from an asset
    pub fn read<Reader: ArchiveReader<impl PackageIndexTrait>>(
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

    /// Write a `TMovieSceneEvaluationTree` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
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

/// Movie scene evaluation tree node
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationTreeNode {
    /// Frame number range
    pub range: FFrameNumberRange,
    /// Parent
    pub parent: MovieSceneEvaluationTreeNodeHandle,
    /// Children id
    pub children_id: EvaluationTreeEntryHandle,
    /// Data id
    pub data_id: EvaluationTreeEntryHandle,
}

impl MovieSceneEvaluationTreeNode {
    /// Read a `MovieSceneEvaluationTreeNode` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
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

    /// Write a `MovieSceneEvaluationTreeNode` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        self.range.write(asset)?;
        self.parent.write(asset)?;
        self.children_id.write(asset)?;
        self.data_id.write(asset)?;

        Ok(())
    }
}

/// Movie entity and metadata index
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct FEntityAndMetaDataIndex {
    /// Entity index
    pub entity_index: i32,
    /// Metadata index
    pub meta_data_index: i32,
}

impl FEntityAndMetaDataIndex {
    /// Read an `FEntityAndMetaDataIndex` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let entity_index = asset.read_i32::<LE>()?;
        let meta_data_index = asset.read_i32::<LE>()?;

        Ok(FEntityAndMetaDataIndex {
            entity_index,
            meta_data_index,
        })
    }

    /// Write an `FEntityAndMetaDataIndex` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        asset.write_i32::<LE>(self.entity_index)?;
        asset.write_i32::<LE>(self.meta_data_index)?;

        Ok(())
    }
}

/// Movie scene evaluation field entity tree
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct MovieSceneEvaluationFieldEntityTree {
    /// Serialized data
    pub serialized_data: TMovieSceneEvaluationTree<FEntityAndMetaDataIndex>,
}

impl MovieSceneEvaluationFieldEntityTree {
    /// Read a `MovieSceneEvaluationFieldEntityTree` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let serialized_data =
            TMovieSceneEvaluationTree::read(asset, |reader| FEntityAndMetaDataIndex::new(reader))?;

        Ok(MovieSceneEvaluationFieldEntityTree { serialized_data })
    }

    /// Write a `MovieSceneEvaluationFieldEntityTree` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        self.serialized_data.write(asset, |writer, node| {
            node.write(writer)?;
            Ok(())
        })?;

        Ok(())
    }
}
