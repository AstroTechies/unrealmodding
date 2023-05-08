//! Color properties

use std::mem::size_of;

use byteorder::LE;
use ordered_float::OrderedFloat;
use unreal_asset_proc_macro::FNameContainer;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::vector::Color;
use crate::types::{fname::FName, Guid};
use crate::unversioned::ancestry::Ancestry;

/// Color property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ColorProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Color
    #[container_ignore]
    pub color: Color<u8>,
}
impl_property_data_trait!(ColorProperty);

/// Linear color property
#[derive(FNameContainer, Debug, Clone, Hash, PartialEq, Eq)]
pub struct LinearColorProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Color
    #[container_ignore]
    pub color: Color<OrderedFloat<f32>>,
}
impl_property_data_trait!(LinearColorProperty);

impl ColorProperty {
    /// Read a `ColorProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let color = Color::from_argb(asset.read_i32::<LE>()?);
        Ok(ColorProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            color,
        })
    }
}

impl PropertyTrait for ColorProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LE>(self.color.to_argb())?;
        Ok(size_of::<i32>())
    }
}

impl LinearColorProperty {
    /// Read a `LinearColorProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let color = Color::new(
            OrderedFloat(asset.read_f32::<LE>()?),
            OrderedFloat(asset.read_f32::<LE>()?),
            OrderedFloat(asset.read_f32::<LE>()?),
            OrderedFloat(asset.read_f32::<LE>()?),
        );
        Ok(LinearColorProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            color,
        })
    }
}

impl PropertyTrait for LinearColorProperty {
    fn write<Writer: ArchiveWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LE>(self.color.r.0)?;
        asset.write_f32::<LE>(self.color.g.0)?;
        asset.write_f32::<LE>(self.color.b.0)?;
        asset.write_f32::<LE>(self.color.a.0)?;
        Ok(size_of::<f32>() * 4)
    }
}
