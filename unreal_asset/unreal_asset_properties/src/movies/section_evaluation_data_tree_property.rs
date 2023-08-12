//! Section evaluation data tree property

use crate::property_prelude::*;

use super::movie_scene_evaluation::TMovieSceneEvaluationTree;

/// Section evaluation tree
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionEvaluationTree {
    /// Evaluation tree
    pub tree: TMovieSceneEvaluationTree<Vec<Property>>,
}

impl SectionEvaluationTree {
    /// Read a `SectionEvaluationTree` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let tree = TMovieSceneEvaluationTree::read(asset, |reader| {
            let mut resulting_list = Vec::new();
            let mut unversioned_header = UnversionedHeader::new(reader)?;
            while let Some(property) = Property::new(
                reader,
                Ancestry::default(),
                unversioned_header.as_mut(),
                true,
            )? {
                resulting_list.push(property);
            }

            Ok(resulting_list)
        })?;

        Ok(SectionEvaluationTree { tree })
    }

    /// Write a `SectionEvaluationTree` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.tree.write(asset, |writer, node| {
            let (unversioned_header, sorted_properties) = match generate_unversioned_header(
                writer,
                node,
                &writer
                    .get_name_map()
                    .get_mut()
                    .add_fname("SectionEvaluationDataTree"),
            )? {
                Some((a, b)) => (Some(a), Some(b)),
                None => (None, None),
            };

            if let Some(unversioned_header) = unversioned_header {
                unversioned_header.write(writer)?;
            }

            let properties = sorted_properties.as_ref().unwrap_or(node);
            for property in properties.iter() {
                Property::write(property, writer, true)?;
            }

            let none_fname = writer.add_fname("None");
            writer.write_fname(&none_fname)?;
            Ok(())
        })?;

        Ok(())
    }
}

/// Section evaluation data tree property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionEvaluationDataTreeProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Evaluation tree
    pub value: SectionEvaluationTree,
}
impl_property_data_trait!(SectionEvaluationDataTreeProperty);

impl SectionEvaluationDataTreeProperty {
    /// Read a `SectionEvaluationDataTreeProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = SectionEvaluationTree::new(asset)?;

        Ok(SectionEvaluationDataTreeProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for SectionEvaluationDataTreeProperty {
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
