use std::mem::size_of;

use byteorder::LittleEndian;

use crate::error::Error;
use crate::impl_property_data_trait;
use crate::optional_guid;
use crate::optional_guid_write;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::unreal_types::{FName, Guid, PackageIndex};

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct Delegate {
    pub object: PackageIndex,
    pub delegate: FName,
}

impl Delegate {
    pub fn new(object: PackageIndex, delegate: FName) -> Self {
        Delegate { object, delegate }
    }
}

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct MulticastDelegateProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vec<Delegate>,
}
impl_property_data_trait!(MulticastDelegateProperty);
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
        for _ in 0..length {
            value.push(Delegate::new(
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

// they all write and read the same so this reduces code duplication
macro_rules! impl_delegate_property {
    ($property_name:ident) => {
        #[derive(Hash, Clone, PartialEq, Eq)]
        pub struct $property_name {
            pub name: FName,
            pub property_guid: Option<Guid>,
            pub duplication_index: i32,
            pub value: Delegate,
        }
        impl_property_data_trait!($property_name);
        impl $property_name {
            pub fn new<Reader: AssetReader>(
                asset: &mut Reader,
                name: FName,
                include_header: bool,
                _length: i64,
                duplication_index: i32,
            ) -> Result<Self, Error> {
                let property_guid = optional_guid!(asset, include_header);

                Ok($property_name {
                    name,
                    property_guid,
                    duplication_index,
                    value: Delegate::new(
                        PackageIndex::new(asset.read_i32::<LittleEndian>()?),
                        asset.read_fname()?,
                    ),
                })
            }
        }

        impl PropertyTrait for $property_name {
            fn write<Writer: AssetWriter>(
                &self,
                asset: &mut Writer,
                include_header: bool,
            ) -> Result<usize, Error> {
                optional_guid_write!(self, asset, include_header);

                asset.write_i32::<LittleEndian>(self.value.object.index)?;
                asset.write_fname(&self.value.delegate)?;
                Ok(size_of::<i32>() * 3)
            }
        }
    };
}

impl_delegate_property!(DelegateProperty);
impl_delegate_property!(MulticastSparseDelegateProperty);
impl_delegate_property!(MulticastInlineDelegateProperty);