use byteorder::LittleEndian;

use crate::{
    error::Error,
    impl_property_data_trait, optional_guid, optional_guid_write,
    reader::asset_reader::AssetReader,
    unreal_types::{FName, Guid, PackageIndex},
};

use super::{PropertyDataTrait, PropertyTrait};

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
pub struct DelegateProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Delegate,
}
impl_property_data_trait!(DelegateProperty);

impl DelegateProperty {
    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        _length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let value = Delegate::new(
            PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            asset.read_fname()?,
        );

        Ok(DelegateProperty {
            name,
            property_guid,
            duplication_index,
            value,
        })
    }
}

impl PropertyTrait for DelegateProperty {
    fn write<Writer: crate::reader::asset_writer::AssetWriter>(
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
