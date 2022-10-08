use std::mem::size_of;

use byteorder::LittleEndian;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, Guid, PackageIndex};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: PackageIndex,
}
impl_property_data_trait!(ObjectProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AssetObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Option<String>,
}
impl_property_data_trait!(AssetObjectProperty);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SoftObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: FName,
    pub id: u32,
}
impl_property_data_trait!(SoftObjectProperty);

impl ObjectProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_i32::<LittleEndian>()?;
        Ok(ObjectProperty {
            name,
            property_guid,
            duplication_index,
            value: PackageIndex::new(value),
        })
    }
}

impl PropertyTrait for ObjectProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_i32::<LittleEndian>(self.value.index)?;
        Ok(size_of::<i32>())
    }
}

impl AssetObjectProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_string()?;
        Ok(AssetObjectProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for AssetObjectProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_string(&self.value)
    }
}

impl SoftObjectProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = asset.read_fname()?;
        let id = asset.read_u32::<LittleEndian>()?;
        Ok(SoftObjectProperty {
            name,
            property_guid,
            duplication_index,
            value,
            id,
        })
    }
}

impl PropertyTrait for SoftObjectProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);
        asset.write_fname(&self.value)?;
        asset.write_u32::<LittleEndian>(self.id)?;
        Ok(size_of::<i32>() * 3)
    }
}
