use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::{core_uobject::FFrameNumberRange, Property, PropertyTrait},
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
    unreal_types::{FName, Guid},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSegmentIdentifier {
    pub identifier_index: i32,
}

impl MovieSceneSegmentIdentifier {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let identifier_index = asset.read_i32::<LittleEndian>()?;

        Ok(MovieSceneSegmentIdentifier { identifier_index })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LittleEndian>(self.identifier_index)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSegment {
    pub range: FFrameNumberRange,
    pub id: MovieSceneSegmentIdentifier,
    pub allow_empty: bool,
    pub impls: Vec<Vec<Property>>,
}

impl MovieSceneSegment {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let range = FFrameNumberRange::new(asset)?;
        let id = MovieSceneSegmentIdentifier::new(asset)?;
        let allow_empty = asset.read_i32::<LittleEndian>()? != 0;

        let impls_length = asset.read_i32::<LittleEndian>()?;
        let mut impls = Vec::with_capacity(impls_length as usize);

        for _ in 0..impls_length {
            let mut properties_list = Vec::new();
            while let Some(property) = Property::new(asset, true)? {
                properties_list.push(property);
            }

            impls.push(properties_list);
        }

        Ok(MovieSceneSegment {
            range,
            id,
            allow_empty,
            impls,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.range.write(asset)?;
        self.id.write(asset)?;

        asset.write_i32::<LittleEndian>(match self.allow_empty {
            true => 1,
            false => 0,
        })?;

        asset.write_i32::<LittleEndian>(self.impls.len() as i32)?;

        let none_fname = asset.add_fname("None");

        for imp in &self.impls {
            for property in imp {
                Property::write(property, asset, true)?;
            }

            asset.write_fname(&none_fname)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSegmentProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneSegment,
}
impl_property_data_trait!(MovieSceneSegmentProperty);

impl MovieSceneSegmentProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneSegment::new(asset)?;

        Ok(MovieSceneSegmentProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneSegmentProperty {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSegmentIdentifierProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: MovieSceneSegmentIdentifier,
}
impl_property_data_trait!(MovieSceneSegmentIdentifierProperty);

impl MovieSceneSegmentIdentifierProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneSegmentIdentifier::new(asset)?;

        Ok(MovieSceneSegmentIdentifierProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneSegmentIdentifierProperty {
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
