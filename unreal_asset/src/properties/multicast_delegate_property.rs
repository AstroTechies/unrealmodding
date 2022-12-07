use std::mem::size_of;

use byteorder::LittleEndian;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::PropertyTrait;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, Guid, PackageIndex};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MulticastDelegate {
    pub object: PackageIndex,
    pub delegate: FName,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct MulticastDelegateProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vec<MulticastDelegate>,
}
impl_property_data_trait!(MulticastDelegateProperty);

impl MulticastDelegate {
    pub fn new(object: PackageIndex, delegate: FName) -> Self {
        MulticastDelegate { object, delegate }
    }
}

impl MulticastDelegateProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);

        let length = asset.read_i32::<LittleEndian>()?;
        let mut value = Vec::with_capacity(length as usize);
        for _i in 0..length as usize {
            value.push(MulticastDelegate::new(
                PackageIndex::new(asset.read_i32::<LittleEndian>()?),
                asset.read_fname()?,
            ));
        }

        Ok(MulticastDelegateProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for MulticastDelegateProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &mut Writer,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, include_header);

        asset.write_i32::<LittleEndian>(self.value.len() as i32)?;
        for entry in &self.value {
            asset.write_i32::<LittleEndian>(entry.object.index)?;
            asset.write_fname(&entry.delegate)?;
        }
        Ok(size_of::<i32>() + size_of::<i32>() * 3 * self.value.len())
    }
}
