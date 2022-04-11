use std::io::Cursor;

use crate::error::{Error, PropertyError};
use crate::properties::{PropertyDataTrait, PropertyTrait};
use crate::unreal_types::ToFName;
use crate::{
    impl_property_data_trait,
    {
        unreal_types::{FName, Guid},
        Asset,
    },
};

use super::array_property::ArrayProperty;

#[derive(Hash, PartialEq, Eq)]
pub struct SetProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub duplication_index: i32,
    pub array_type: Option<FName>,
    pub value: ArrayProperty,
    pub removed_items: ArrayProperty,
}
impl_property_data_trait!(SetProperty);

impl SetProperty {
    pub fn new(
        asset: &mut Asset,
        name: FName,
        include_header: bool,
        length: i64,
        duplication_index: i32,
        engine_version: i32,
    ) -> Result<Self, Error> {
        let (array_type, property_guid) = match include_header {
            true => (Some(asset.read_fname()?), asset.read_property_guid()?),
            false => (None, None),
        };

        let removed_items = ArrayProperty::new_no_header(
            asset,
            name.clone(),
            false,
            length,
            0,
            engine_version,
            false,
            array_type.clone(),
            property_guid,
        )?;

        let items = ArrayProperty::new_no_header(
            asset,
            name.clone(),
            false,
            length,
            0,
            engine_version,
            false,
            array_type.clone(),
            property_guid,
        )?;

        Ok(SetProperty {
            name,
            property_guid,
            duplication_index,
            array_type,
            value: items,
            removed_items,
        })
    }
}

impl PropertyTrait for SetProperty {
    fn write(
        &self,
        asset: &Asset,
        cursor: &mut Cursor<Vec<u8>>,
        include_header: bool,
    ) -> Result<usize, Error> {
        let array_type = match self.value.value.len() > 0 {
            true => Some(self.value.value[0].to_fname()),
            false => self.array_type.clone(),
        };

        if include_header {
            asset.write_fname(
                cursor,
                array_type.as_ref().ok_or(PropertyError::headerless())?,
            )?;
            asset.write_property_guid(cursor, &self.property_guid)?;
        }

        let removed_items_len = self.removed_items.write_full(asset, cursor, false, false)?;
        let items_len = self.value.write_full(asset, cursor, false, false)?;
        Ok(removed_items_len + items_len)
    }
}
