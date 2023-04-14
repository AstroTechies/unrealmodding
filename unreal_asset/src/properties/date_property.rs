//! Date properties

use std::mem::size_of;

use byteorder::LittleEndian;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::simple_property_write;
use crate::types::{FName, Guid};

/// Time span property
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TimeSpanProperty {
    /// Name
    pub name: FName,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Ticks
    pub ticks: i64,
}
impl_property_data_trait!(TimeSpanProperty);

/// Date time property
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DateTimeProperty {
    /// Name
    pub name: FName,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Ticks
    pub ticks: i64,
}
impl_property_data_trait!(DateTimeProperty);

impl TimeSpanProperty {
    /// Read a `TimeSpanProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let ticks = asset.read_i64::<LittleEndian>()?;
        Ok(TimeSpanProperty {
            name,
            property_guid,
            duplication_index,
            ticks,
        })
    }
}

simple_property_write!(TimeSpanProperty, write_i64, ticks, i64);

impl DateTimeProperty {
    /// Read a `DateTimeProperty` from an asset
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let ticks = asset.read_i64::<LittleEndian>()?;
        Ok(DateTimeProperty {
            name,
            property_guid,
            duplication_index,
            ticks,
        })
    }
}

simple_property_write!(DateTimeProperty, write_i64, ticks, i64);
