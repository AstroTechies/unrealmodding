use std::io::{Cursor};
use crate::uasset::Asset;
use crate::uasset::exports::unknown_export::UnknownExport;
use crate::uasset::properties::Property;
use crate::uasset::unreal_types::{FName};
use crate::uasset::error::Error;
use crate::uasset::exports::{ExportTrait, ExportUnknownTrait};

use super::ExportNormalTrait;

pub struct NormalExport {
    pub unknown_export: UnknownExport,
    pub extras: Vec<u8>,

    pub properties: Vec<Property>
}

impl ExportNormalTrait for NormalExport {
    fn get_normal_export< 'a>(&'a self) -> Option<& 'a NormalExport> {
        Some(&self)
    }


    fn get_normal_export_mut< 'a>(&'a mut self) -> Option<& 'a mut NormalExport> {
        Some(self)
    }

}

impl ExportUnknownTrait for NormalExport {
    fn get_unknown_export<'a>(&'a self) -> &'a UnknownExport {
        &self.unknown_export
    }

    fn get_unknown_export_mut<'a>(&'a mut self) -> &'a mut UnknownExport {
        &mut self.unknown_export
    }
}

impl NormalExport {
    pub fn from_unk(unk: &UnknownExport, asset: &mut Asset) -> Result<Self, Error> {
        let _cursor = &mut asset.cursor;
        let mut properties = Vec::new();

        while let Some(e) = Property::new(asset, true)? {
            properties.push(e);
        }

        Ok(NormalExport {
            unknown_export: unk.clone(),
            extras: Vec::new(),

            properties
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