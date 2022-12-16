use super::movie_scene_evaluation::TMovieSceneEvaluationTree;
use crate::error::Error;
use crate::properties::{Property, PropertyTrait};
use crate::reader::asset_reader::AssetReader;
use crate::reader::asset_writer::AssetWriter;
use crate::unreal_types::{FName, Guid};
use crate::{impl_property_data_trait, optional_guid, optional_guid_write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionEvaluationTree {
    pub tree: TMovieSceneEvaluationTree<Vec<Property>>,
}

impl SectionEvaluationTree {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let tree = TMovieSceneEvaluationTree::read(asset, |reader| {
            let mut resulting_list = Vec::new();
            while let Some(property) = Property::new(reader, true)? {
                resulting_list.push(property);
            }

            Ok(resulting_list)
        })?;

        Ok(SectionEvaluationTree { tree })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.tree.write(asset, |writer, node| {
            for property in node {
                Property::write(property, writer, true)?;
            }

            let none_fname = writer.add_fname("None");
            writer.write_fname(&none_fname)?;
            Ok(())
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionEvaluationDataTreeProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: SectionEvaluationTree,
}
impl_property_data_trait!(SectionEvaluationDataTreeProperty);

impl SectionEvaluationDataTreeProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = SectionEvaluationTree::new(asset)?;

        Ok(SectionEvaluationDataTreeProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for SectionEvaluationDataTreeProperty {
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
