use std::io::Cursor;
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    types::Color,
    unreal_types::{FName, Guid},
    Asset,
};

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
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let color = Color::from_argb(asset.cursor.read_i32::<LittleEndian>()?);
        Ok(ColorProperty {
            name,
            property_guid,
            duplication_index,
            color,
        })
    }
}

impl PropertyTrait for ColorProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i32::<LittleEndian>(self.color.to_argb())?;
        Ok(size_of::<i32>())
    }
}

impl LinearColorProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let color = Color::new(
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
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
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f32::<LittleEndian>(self.color.r.0)?;
        cursor.write_f32::<LittleEndian>(self.color.g.0)?;
        cursor.write_f32::<LittleEndian>(self.color.b.0)?;
        cursor.write_f32::<LittleEndian>(self.color.a.0)?;
        Ok(size_of::<f32>() * 4)
    }
}
