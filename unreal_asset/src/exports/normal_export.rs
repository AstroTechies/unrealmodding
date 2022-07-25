use crate::asset_reader::AssetReader;
use crate::asset_writer::AssetWriter;
use crate::error::Error;
use crate::exports::base_export::BaseExport;
use crate::exports::{ExportBaseTrait, ExportTrait};
use crate::properties::Property;
use crate::unreal_types::FName;
use std::io::Cursor;

use super::ExportNormalTrait;

#[derive(Clone)]
pub struct NormalExport {
    pub base_export: BaseExport,
    pub extras: Vec<u8>,

    pub properties: Vec<Property>,
}

impl ExportNormalTrait for NormalExport {
    fn get_normal_export(&'_ self) -> Option<&'_ NormalExport> {
        Some(self)
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut NormalExport> {
        Some(self)
    }
}

impl ExportBaseTrait for NormalExport {
    fn get_base_export(&'_ self) -> &'_ BaseExport {
        &self.base_export
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport {
        &mut self.base_export
    }
}

impl NormalExport {
    pub fn from_base<Reader: AssetReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let mut properties = Vec::new();
        while let Some(e) = Property::new(asset, true)? {
            properties.push(e);
        }

        Ok(NormalExport {
            base_export: base.clone(),
            extras: Vec::new(),

            properties,
        })
    }
}

impl ExportTrait for NormalExport {
    fn write<Writer: AssetWriter>(
        &self,
        asset: &Writer,
        cursor: &mut Cursor<Vec<u8>>,
    ) -> Result<(), Error> {
        for entry in &self.properties {
            Property::write(entry, asset, cursor, true)?;
        }
        asset.write_fname(cursor, &FName::from_slice("None"))?;
        Ok(())
    }
}
