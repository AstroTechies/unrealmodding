//! Movie scene segment property

use crate::properties::property_prelude::*;

use unreal_asset_proc_macro::FNameContainer;
use unreal_helpers::Guid;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::generate_unversioned_header,
    properties::{Property, PropertyTrait},
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    types::fname::FName,
    types::movie::FFrameNumberRange,
    unversioned::{ancestry::Ancestry, header::UnversionedHeader},
};

/// Movie scene segment identifier
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSegmentIdentifier {
    /// Identifier index
    pub identifier_index: i32,
}

impl MovieSceneSegmentIdentifier {
    /// Read a `MovieSceneSegmentIdentifier` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let identifier_index = asset.read_i32::<LE>()?;

        Ok(MovieSceneSegmentIdentifier { identifier_index })
    }

    /// Write a `MovieSceneSegmentIdentifier` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i32::<LE>(self.identifier_index)?;
        Ok(())
    }
}

/// Movie scene segment
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSegment {
    /// Name
    pub name: FName,
    /// range
    #[container_ignore]
    pub range: FFrameNumberRange,
    /// Identifier
    #[container_ignore]
    pub id: MovieSceneSegmentIdentifier,
    /// Allow empty
    pub allow_empty: bool,
    /// Implementations
    pub impls: Vec<Vec<Property>>,
}

impl MovieSceneSegment {
    /// Read a `MovieSceneSegment` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
    ) -> Result<Self, Error> {
        let range = FFrameNumberRange::new(asset)?;
        let id = MovieSceneSegmentIdentifier::new(asset)?;
        let allow_empty = asset.read_i32::<LE>()? != 0;

        let impls_length = asset.read_i32::<LE>()?;
        let mut impls = Vec::with_capacity(impls_length as usize);

        for _ in 0..impls_length {
            let mut properties_list = Vec::new();
            let mut unversioned_header = UnversionedHeader::new(asset)?;
            while let Some(property) =
                Property::new(asset, ancestry.clone(), unversioned_header.as_mut(), true)?
            {
                properties_list.push(property);
            }

            impls.push(properties_list);
        }

        Ok(MovieSceneSegment {
            name,
            range,
            id,
            allow_empty,
            impls,
        })
    }

    /// Write a `MovieSceneSegment` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.range.write(asset)?;
        self.id.write(asset)?;

        asset.write_i32::<LE>(match self.allow_empty {
            true => 1,
            false => 0,
        })?;

        asset.write_i32::<LE>(self.impls.len() as i32)?;

        let none_fname = asset.add_fname("None");

        for imp in &self.impls {
            let (unversioned_header, sorted_properties) =
                match generate_unversioned_header(asset, imp, &self.name)? {
                    Some((a, b)) => (Some(a), Some(b)),
                    None => (None, None),
                };

            if let Some(unversioned_header) = unversioned_header {
                unversioned_header.write(asset)?;
            }

            let properties = sorted_properties.as_ref().unwrap_or(imp);
            for property in properties.iter() {
                Property::write(property, asset, true)?;
            }

            asset.write_fname(&none_fname)?;
        }

        Ok(())
    }
}

/// Movie scene segment property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSegmentProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Value
    pub value: MovieSceneSegment,
}
impl_property_data_trait!(MovieSceneSegmentProperty);

impl MovieSceneSegmentProperty {
    /// Read a `MovieSceneSegmentProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value =
            MovieSceneSegment::new(asset, name.clone(), ancestry.with_parent(name.clone()))?;

        Ok(MovieSceneSegmentProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneSegmentProperty {
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

/// Movie scene segment identifier property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneSegmentIdentifierProperty {
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
    pub value: MovieSceneSegmentIdentifier,
}
impl_property_data_trait!(MovieSceneSegmentIdentifierProperty);

impl MovieSceneSegmentIdentifierProperty {
    /// Read a `MovieSceneSegmentIdentifierProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let value = MovieSceneSegmentIdentifier::new(asset)?;

        Ok(MovieSceneSegmentIdentifierProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MovieSceneSegmentIdentifierProperty {
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
