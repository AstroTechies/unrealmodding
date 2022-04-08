use std::io::{Cursor, ErrorKind, Read};
use std::mem::size_of;


use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid, optional_guid_write, simple_property_write};
use crate::uasset::properties::PropertyTrait;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TimeSpanProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub ticks: i64
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DateTimeProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub ticks: i64
}

impl TimeSpanProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let ticks = asset.cursor.read_i64::<LittleEndian>()?;
        Ok(TimeSpanProperty {
            name,
            property_guid,
            ticks
        })
    }
}

simple_property_write!(TimeSpanProperty, write_i64, ticks, i64);

impl DateTimeProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let ticks = asset.cursor.read_i64::<LittleEndian>()?;
        Ok(DateTimeProperty {
            name,
            property_guid,
            ticks
        })
    }
}

simple_property_write!(DateTimeProperty, write_i64, ticks, i64);