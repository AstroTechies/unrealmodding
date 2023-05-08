//! Movie scene frame range property

use byteorder::LE;
use unreal_asset_proc_macro::FNameContainer;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    properties::PropertyTrait,
    reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter},
    types::movie::ERangeBoundTypes,
    types::{fname::FName, Guid},
    unversioned::ancestry::Ancestry,
};

/// Int32 value bound by a range
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Int32RangeBound {
    /// Type
    pub ty: ERangeBoundTypes,
    /// Value
    pub value: i32,
}

impl Int32RangeBound {
    /// Read an `Int32RangeBound` from an asset
    pub fn new<Reader: ArchiveReader>(asset: &mut Reader) -> Result<Self, Error> {
        let ty: ERangeBoundTypes = ERangeBoundTypes::try_from(asset.read_i8()?)?;
        let value = asset.read_i32::<LE>()?;

        Ok(Int32RangeBound { ty, value })
    }

    /// Write an `Int32RangeBound` to an asset
    pub fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_i8(self.ty as i8)?;
        asset.write_i32::<LE>(self.value)?;

        Ok(())
    }
}

/// Movie scene frame range property
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovieSceneFrameRangeProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Lower bound
    #[container_ignore]
    pub lower_bound: Int32RangeBound,
    /// Upper bound
    #[container_ignore]
    pub upper_bound: Int32RangeBound,
}
impl_property_data_trait!(MovieSceneFrameRangeProperty);

impl MovieSceneFrameRangeProperty {
    /// Read a `MovieSceneFrameRangeProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let lower_bound = Int32RangeBound::new(asset)?;
        let upper_bound = Int32RangeBound::new(asset)?;

        Ok(MovieSceneFrameRangeProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            lower_bound,
            upper_bound,
        })
    }
}

impl PropertyTrait for MovieSceneFrameRangeProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        let begin = asset.position();

        self.lower_bound.write(asset)?;
        self.upper_bound.write(asset)?;

        Ok((asset.position() - begin) as usize)
    }
}
