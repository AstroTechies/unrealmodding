//! Date properties

use unreal_asset_base::types::PackageIndexTrait;

use crate::property_prelude::*;

/// Time span property
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct TimeSpanProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
    /// Property guid
    pub property_guid: Option<Guid>,
    /// Property duplication index
    pub duplication_index: i32,
    /// Ticks
    pub ticks: i64,
}
impl_property_data_trait!(TimeSpanProperty);

/// Date time property
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DateTimeProperty {
    /// Name
    pub name: FName,
    /// Property ancestry
    pub ancestry: Ancestry,
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
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let ticks = asset.read_i64::<LE>()?;
        Ok(TimeSpanProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            ticks,
        })
    }
}

simple_property_write!(TimeSpanProperty, write_i64, ticks, i64);

impl DateTimeProperty {
    /// Read a `DateTimeProperty` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        name: FName,
        ancestry: Ancestry,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let ticks = asset.read_i64::<LE>()?;
        Ok(DateTimeProperty {
            name,
            ancestry,
            property_guid,
            duplication_index,
            ticks,
        })
    }
}

simple_property_write!(DateTimeProperty, write_i64, ticks, i64);
