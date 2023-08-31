//! Function export

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    flags::EFunctionFlags,
    reader::{ArchiveReader, ArchiveWriter},
    types::PackageIndexTrait,
    Error, FNameContainer,
};

use crate::{BaseExport, StructExport};
use crate::{ExportBaseTrait, ExportNormalTrait, ExportTrait};

/// Function export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionExport<Index: PackageIndexTrait> {
    /// Base struct export
    pub struct_export: StructExport<Index>,
    /// Function flags
    #[container_ignore]
    pub function_flags: EFunctionFlags,
}

impl<Index: PackageIndexTrait> FunctionExport<Index> {
    /// Read a `FunctionExport` from an asset
    pub fn from_base<Reader: ArchiveReader<Index>>(
        base: &BaseExport<Index>,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let struct_export = StructExport::from_base(base, asset)?;
        let function_flags = EFunctionFlags::from_bits(asset.read_u32::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid function flags".to_string()))?;
        Ok(FunctionExport {
            struct_export,
            function_flags,
        })
    }
}

impl<Index: PackageIndexTrait> ExportTrait<Index> for FunctionExport<Index> {
    fn write<Writer: ArchiveWriter<Index>>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.struct_export.write(asset)?;
        asset.write_u32::<LE>(self.function_flags.bits())?;
        Ok(())
    }
}

impl<Index: PackageIndexTrait> ExportNormalTrait<Index> for FunctionExport<Index> {
    fn get_normal_export(&'_ self) -> Option<&'_ super::normal_export::NormalExport<Index>> {
        self.struct_export.get_normal_export()
    }

    fn get_normal_export_mut(
        &'_ mut self,
    ) -> Option<&'_ mut super::normal_export::NormalExport<Index>> {
        self.struct_export.get_normal_export_mut()
    }
}

impl<Index: PackageIndexTrait> ExportBaseTrait<Index> for FunctionExport<Index> {
    fn get_base_export(&'_ self) -> &'_ super::base_export::BaseExport<Index> {
        self.struct_export.get_base_export()
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut super::base_export::BaseExport<Index> {
        self.struct_export.get_base_export_mut()
    }
}
