use std::mem::size_of;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::{FName, Guid};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PerPlatformBoolProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vec<bool>,
}
impl_property_data_trait!(PerPlatformBoolProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PerPlatformIntProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vec<i32>,
}
impl_property_data_trait!(PerPlatformIntProperty);

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PerPlatformFloatProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vec<OrderedFloat<f32>>,
}
impl_property_data_trait!(PerPlatformFloatProperty);

impl PerPlatformBoolProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let num_entries = asset.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(num_entries as usize);

        for _i in 0..num_entries as usize {
            value.push(asset.read_bool()?);
        }

        Ok(PerPlatformBoolProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for PerPlatformBoolProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LittleEndian>(self.value.len() as i32)?;
        for entry in &self.value {
            asset.write_bool(*entry)?;
        }
        Ok(size_of::<i32>() + size_of::<bool>() * self.value.len())
    }
}

impl PerPlatformIntProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let num_entries = asset.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(num_entries as usize);

        for _i in 0..num_entries as usize {
            value.push(asset.read_i32::<LittleEndian>()?);
        }

        Ok(PerPlatformIntProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for PerPlatformIntProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LittleEndian>(self.value.len() as i32)?;
        for entry in &self.value {
            asset.write_i32::<LittleEndian>(*entry)?;
        }
        Ok(size_of::<i32>() + size_of::<i32>() * self.value.len())
    }
}

impl PerPlatformFloatProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let num_entries = asset.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(num_entries as usize);

        for _i in 0..num_entries as usize {
            value.push(OrderedFloat(asset.read_f32::<LittleEndian>()?));
        }

        Ok(PerPlatformFloatProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for PerPlatformFloatProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LittleEndian>(self.value.len() as i32)?;
        for entry in &self.value {
            asset.write_f32::<LittleEndian>(entry.0)?;
        }
        Ok(size_of::<i32>() + size_of::<f32>() * self.value.len())
    }
}
