use std::io::{Cursor, Write};

use crate::asset_reader::AssetReader;
use crate::asset_writer::AssetWriter;
use crate::error::Error;
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::{
    impl_property_data_trait, optional_guid, optional_guid_write,
    unreal_types::{FName, Guid},
};

#[derive(Hash, Clone, PartialEq, Eq)]
pub struct UnknownProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub value: Vec<u8>,
    pub serialized_type: Option<FName>,
}
impl_property_data_trait!(UnknownProperty);

impl UnknownProperty {
    pub fn with_serialized_type<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        length: i64,
        duplication_index: i32,
        serialized_type: Option<FName>,
    ) -> Result<Self, Error> {
        let property_guid = optional_guid!(asset, include_header);
        let mut value = vec![0u8; length as usize];
        asset.read_exact(&mut value)?;

        Ok(UnknownProperty {
            name,
            property_guid,
            duplication_index,
            value,
            serialized_type,
        })
    }

    pub fn new<Reader: AssetReader>(
        asset: &mut Reader,
        name: FName,
        include_header: bool,
        length: i64,
        duplication_index: i32,
    ) -> Result<Self, Error> {
        UnknownProperty::with_serialized_type(
            asset,
            name,
            include_header,
            length,
            duplication_index,
            None,
        )
    }
}

impl PropertyTrait for UnknownProperty {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &Writer,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        optional_guid_write!(self, asset, cursor, include_header);
        cursor.write_all(&self.value)?;
        Ok(self.value.len())
    }
}
