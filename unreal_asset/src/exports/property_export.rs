use crate::asset_reader::AssetReader;
use crate::asset_writer::AssetWriter;
use crate::exports::base_export::BaseExport;
use crate::exports::normal_export::NormalExport;
use crate::exports::ExportTrait;
use crate::implement_get;
use crate::uproperty::{UProperty, UPropertyTrait};
use crate::Error;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Cursor;

use super::ExportBaseTrait;
use super::ExportNormalTrait;

#[derive(Clone)]
pub struct PropertyExport {
    pub normal_export: NormalExport,

    pub property: UProperty,
}

implement_get!(PropertyExport);

impl PropertyExport {
    pub fn from_base<Reader: AssetReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(base, asset)?;

        asset.read_i32::<LittleEndian>()?;

        let export_class_type = asset
            .get_export_class_type(normal_export.base_export.class_index)
            .ok_or_else(|| Error::invalid_package_index("No such class type".to_string()))?;
        let property = UProperty::new(asset, export_class_type)?;

        Ok(PropertyExport {
            normal_export,
            property,
        })
    }
}

impl ExportTrait for PropertyExport {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &Writer,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<(), Error> {
        self.normal_export.write(asset, cursor)?;
        cursor.write_i32::<LittleEndian>(0)?;
        self.property.write(asset, cursor)?;
        Ok(())
    }
}
