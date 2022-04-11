use std::io::{Cursor,};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::implement_get;
use crate::uasset::Asset;
use crate::uasset::Error;
use crate::uasset::exports::ExportTrait;
use crate::uasset::exports::normal_export::NormalExport;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::uproperty::{UProperty, UPropertyTrait};

use super::ExportNormalTrait;
use super::ExportUnknownTrait;

pub struct PropertyExport {
    pub normal_export: NormalExport,

    pub property: UProperty
}

implement_get!(PropertyExport);

impl PropertyExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let normal_export = NormalExport::from_unk(unk, asset)?;

        asset.cursor.read_i32::<LittleEndian>()?;

        let export_class_type = asset.get_export_class_type(normal_export.unknown_export.class_index).ok_or(Error::invalid_package_index("No such class type".to_string()))?;
        let property = UProperty::new(asset, export_class_type)?;

        Ok(PropertyExport {
            normal_export,
            property
        })
    }
}

impl ExportTrait for PropertyExport {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;
        cursor.write_i32::<LittleEndian>(0)?;
        self.property.write(asset, cursor)?;
        Ok(())
    }
}