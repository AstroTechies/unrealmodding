use std::io::{Cursor};

use byteorder::{LittleEndian, ReadBytesExt};
use crate::uasset::error::Error;
use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct MulticastDelegate {
    number: i32,
    delegate: FName
}

#[derive(Hash, PartialEq, Eq)]
pub struct MulticastDelegateProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vec<MulticastDelegate>
}

impl MulticastDelegate {
    pub fn new(number: i32, delegate: FName) -> Self {
        MulticastDelegate { number, delegate }
    }
}

impl MulticastDelegateProperty {
    pub fn new(asset: &mut Asset, name: FName, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let length = asset.cursor.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(length as usize);
        for i in 0..length as usize {
            value.push(MulticastDelegate::new(asset.cursor.read_i32::<LittleEndian>()?, asset.read_fname()?));
        }

        Ok(MulticastDelegateProperty {
            name,
            property_guid,
            value
        })
    }
}
