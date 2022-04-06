use std::io::{Cursor, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};
use ordered_float::OrderedFloat;

use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct PerPlatformBoolProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vec<bool>
}

impl PerPlatformBoolProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(num_entries as usize);

        for i in 0..num_entries as usize {
            value.push(asset.cursor.read_bool()?);
        }

        Ok(PerPlatformBoolProperty {
            name,
            property_guid,
            value
        })
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct PerPlatformIntProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vec<i32>
}

impl PerPlatformIntProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(num_entries as usize);

        for i in 0..num_entries as usize {
            value.push(asset.cursor.read_i32::<LittleEndian>()?);
        }

        Ok(PerPlatformIntProperty {
            name,
            property_guid,
            value
        })
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct PerPlatformFloatProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vec<OrderedFloat<f32>>
}

impl PerPlatformFloatProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(num_entries as usize);

        for i in 0..num_entries as usize {
            value.push(OrderedFloat(asset.cursor.read_f32::<LittleEndian>()?));
        }

        Ok(PerPlatformFloatProperty {
            name,
            property_guid,
            value
        })
    }
}
