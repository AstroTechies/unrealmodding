use byteorder::LittleEndian;

use crate::{
    error::Error, flags::EFunctionFlags, reader::asset_reader::AssetReader,
    reader::asset_writer::AssetWriter,
};

use super::{
    base_export::BaseExport, struct_export::StructExport, ExportBaseTrait, ExportNormalTrait,
    ExportTrait,
};

#[derive(Clone)]
pub struct FunctionExport {
    pub struct_export: StructExport,
    pub function_flags: EFunctionFlags,
}

impl FunctionExport {
    pub fn from_base<Reader: AssetReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let struct_export = StructExport::from_base(base, asset)?;
        let function_flags = EFunctionFlags::from_bits(asset.read_u32::<LittleEndian>()?)
            .ok_or_else(|| Error::invalid_file("Invalid function flags".to_string()))?;
        Ok(FunctionExport {
            struct_export,
            function_flags,
        })
    }
}

impl ExportTrait for FunctionExport {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.struct_export.write(asset)?;
        asset.write_u32::<LittleEndian>(self.function_flags.bits())?;
        Ok(())
    }
}

impl ExportNormalTrait for FunctionExport {
    fn get_normal_export(&'_ self) -> Option<&'_ super::normal_export::NormalExport> {
        self.struct_export.get_normal_export()
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut super::normal_export::NormalExport> {
        self.struct_export.get_normal_export_mut()
    }
}

impl ExportBaseTrait for FunctionExport {
    fn get_base_export(&'_ self) -> &'_ super::base_export::BaseExport {
        self.struct_export.get_base_export()
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut super::base_export::BaseExport {
        self.struct_export.get_base_export_mut()
    }
}
