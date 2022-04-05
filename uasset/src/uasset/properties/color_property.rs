use std::io::{Cursor, Error, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use ordered_float::OrderedFloat;

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, types::Color, Asset}, optional_guid};

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
