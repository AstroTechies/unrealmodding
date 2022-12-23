use std::mem::size_of;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::vector::Color;
use crate::types::{FName, Guid};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ColorProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub color: Color<u8>,
}
impl_property_data_trait!(ColorProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LinearColorProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub color: Color<OrderedFloat<f32>>,
}
impl_property_data_trait!(LinearColorProperty);

impl ColorProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let color = Color::from_argb(asset.read_i32::<LittleEndian>()?);
        Ok(ColorProperty {
            name,
            property_guid,
            duplication_index,
            color,
        })
    }
}

impl PropertyTrait for ColorProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LittleEndian>(self.color.to_argb())?;
        Ok(size_of::<i32>())
    }
}

impl LinearColorProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let color = Color::new(
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
        );
        Ok(LinearColorProperty {
            name,
            property_guid,
            duplication_index,
            color,
        })
    }
}

impl PropertyTrait for LinearColorProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_f32::<LittleEndian>(self.color.r.0)?;
        asset.write_f32::<LittleEndian>(self.color.g.0)?;
        asset.write_f32::<LittleEndian>(self.color.b.0)?;
        asset.write_f32::<LittleEndian>(self.color.a.0)?;
        Ok(size_of::<f32>() * 4)
    }
}
