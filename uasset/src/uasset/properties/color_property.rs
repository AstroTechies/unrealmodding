use std::io::{Cursor, ErrorKind, Read};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use ordered_float::OrderedFloat;

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, types::Color, Asset}, optional_guid, optional_guid_write};
use crate::uasset::properties::PropertyTrait;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ColorProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub color: Color<u8>
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct LinearColorProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub color: Color<OrderedFloat<f32>>
}

impl ColorProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let color = Color::from_argb(asset.cursor.read_i32::<LittleEndian>()?);
        Ok(ColorProperty {
            name,
            property_guid,
            color
        })
    }
}

impl PropertyTrait for ColorProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_i32::<LittleEndian>(self.color.to_argb())?;
        Ok(size_of::<i32>())
    }
}

impl LinearColorProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let color = Color::new(
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?)
        );
        Ok(LinearColorProperty {
            name,
            property_guid,
            color
        })
    }
}

impl PropertyTrait for LinearColorProperty {
    fn write(&self, asset: &mut Asset, cursor: &mut Cursor<Vec<u8>>, include_header: bool) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_f32::<LittleEndian>(self.color.r.0)?;
        cursor.write_f32::<LittleEndian>(self.color.g.0)?;
        cursor.write_f32::<LittleEndian>(self.color.b.0)?;
        cursor.write_f32::<LittleEndian>(self.color.a.0)?;
        Ok(size_of::<f32>() * 4)
    }
}