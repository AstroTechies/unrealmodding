//! Raw export

use unreal_asset_base::{
    reader::{ArchiveReader, ArchiveWriter},
    types::PackageIndexTrait,
    Error, FNameContainer,
};

use crate::BaseExport;
use crate::{ExportBaseTrait, ExportNormalTrait, ExportTrait};

/// An export that failed to deserialize is stored as `Vec<u8>`
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawExport<Index: PackageIndexTrait> {
    /// Base export
    pub base_export: BaseExport<Index>,
    /// Raw data
    pub data: Vec<u8>,
}

impl<Index: PackageIndexTrait> ExportNormalTrait<Index> for RawExport<Index> {
    fn get_normal_export(&'_ self) -> Option<&'_ super::normal_export::NormalExport<Index>> {
        None
    }

    fn get_normal_export_mut(
        &'_ mut self,
    ) -> Option<&'_ mut super::normal_export::NormalExport<Index>> {
        None
    }
}

impl<Index: PackageIndexTrait> ExportBaseTrait<Index> for RawExport<Index> {
    fn get_base_export(&'_ self) -> &'_ BaseExport<Index> {
        &self.base_export
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport<Index> {
        &mut self.base_export
    }
}

impl<Index: PackageIndexTrait> RawExport<Index> {
    /// Read `RawExport` from an asset
    pub fn from_base<Reader: ArchiveReader<impl PackageIndexTrait>>(
        base: BaseExport<Index>,
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

impl<Index: PackageIndexTrait> ExportTrait<Index> for RawExport<Index> {
    fn write<Writer: ArchiveWriter<Index>>(&self, asset: &mut Writer) -> Result<(), Error> {
        asset.write_all(&self.data)?;
        Ok(())
    }
}
