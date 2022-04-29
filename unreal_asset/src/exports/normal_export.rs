use crate::error::Error;
use crate::exports::base_export::BaseExport;
use crate::exports::{ExportBaseTrait, ExportTrait};
use crate::properties::Property;
use crate::unreal_types::FName;
use crate::Asset;
use std::io::Cursor;

use super::ExportNormalTrait;

#[derive(Clone)]
pub struct NormalExport {
    pub base_export: BaseExport,
    pub extras: Vec<u8>,

    pub properties: Vec<Property>,
}

impl ExportNormalTrait for NormalExport {
    fn get_normal_export<'a>(&'a self) -> Option<&'a NormalExport> {
        Some(&self)
    }

    fn get_normal_export_mut<'a>(&'a mut self) -> Option<&'a mut NormalExport> {
        Some(self)
    }
}

impl ExportBaseTrait for NormalExport {
    fn get_base_export<'a>(&'a self) -> &'a BaseExport {
        &self.base_export
    }

    fn get_base_export_mut<'a>(&'a mut self) -> &'a mut BaseExport {
        &mut self.base_export
    }
}

impl NormalExport {
    pub fn from_base(base: &BaseExport, asset: &mut Asset) -> Result<Self, Error> {
        let _cursor = &mut asset.cursor;
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
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<(), Error> {
        for entry in &self.properties {
            Property::write(entry, asset, cursor, true)?;
        }
        asset.write_fname(cursor, &FName::from_slice("None"))?;
        Ok(())
    }
}
