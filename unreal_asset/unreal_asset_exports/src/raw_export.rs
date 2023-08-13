//! Raw export

use unreal_asset_proc_macro::FNameContainer;

use crate::reader::{ArchiveReader, ArchiveWriter};
use crate::Error;
use crate::{base_export::BaseExport, ExportBaseTrait, ExportNormalTrait, ExportTrait};

/// An export that failed to deserialize is stored as `Vec<u8>`
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawExport {
    /// Base export
    pub base_export: BaseExport,
    /// Raw data
    pub data: Vec<u8>,
}

impl ExportNormalTrait for RawExport {
    fn get_normal_export(&'_ self) -> Option<&'_ super::normal_export::NormalExport> {
        None
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut super::normal_export::NormalExport> {
        None
    }
}

impl ExportBaseTrait for RawExport {
    fn get_base_export(&'_ self) -> &'_ BaseExport {
        &self.base_export
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport {
        &mut self.base_export
    }
}

impl RawExport {
    /// Read `RawExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
        base: BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let mut data = vec![0u8; base.serial_size as usize];
        asset.read_exact(&mut data)?;

        Ok(RawExport {
            base_export: base,
            data,
        })
    }
}

impl ExportTrait for RawExport {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_all(&self.data)?;
        Ok(())
    }
}
