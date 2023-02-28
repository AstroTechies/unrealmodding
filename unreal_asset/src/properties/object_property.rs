use std::mem::size_of;

use byteorder::LittleEndian;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::{FName, Guid, PackageIndex};

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
pub struct SoftObjectPath {
    pub asset_path_name: FName,
    pub sub_path_string: Option<String>,
}

impl SoftObjectPath {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let asset_path_name = asset.read_fname()?;
        let sub_path_string = asset.read_fstring()?;

        Ok(SoftObjectPath {
            asset_path_name,
            sub_path_string,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_fname(&self.asset_path_name)?;
        asset.write_fstring(self.sub_path_string.as_deref())?;

        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SoftObjectProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: SoftObjectPath,
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
        let value = asset.read_fstring()?;
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
        Ok(asset.write_fstring(self.value.as_deref())?)
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
        let value = SoftObjectPath::new(asset)?;

        Ok(SoftObjectProperty {
            name,
            property_guid,
            duplication_index,
            value,
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

        let begin = asset.position();

        self.value.write(asset)?;

        Ok((asset.position() - begin) as usize)
    }
}
