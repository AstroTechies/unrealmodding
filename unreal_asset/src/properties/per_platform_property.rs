//! Per platform properties

use std::mem::size_of;

use byteorder::LittleEndian;
use ordered_float::OrderedFloat;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::types::{FName, Guid};
use crate::unversioned::ancestry::Ancestry;

/// Per platform bool property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PerPlatformBoolProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Values for each platform
    pub value: Vec<bool>,
}
impl_property_data_trait!(PerPlatformBoolProperty);

/// Per platform int property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PerPlatformIntProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Values for each platform
    pub value: Vec<i32>,
}
impl_property_data_trait!(PerPlatformIntProperty);

/// Per platform float property
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct PerPlatformFloatProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Values for each platform
    pub value: Vec<OrderedFloat<f32>>,
}
impl_property_data_trait!(PerPlatformFloatProperty);

impl PerPlatformBoolProperty {
    /// Read a `PerPlatformBoolProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
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
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for PerPlatformBoolProperty {
    fn write<Writer: ArchiveWriter>(
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
    /// Read a `PerPlatformIntProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
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
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for PerPlatformIntProperty {
    fn write<Writer: ArchiveWriter>(
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
    /// Read a `PerPlatformFloatProperty` from an asset
    pub fn new<Reader: ArchiveReader>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
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
            ancestry,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for PerPlatformFloatProperty {
    fn write<Writer: ArchiveWriter>(
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
