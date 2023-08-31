//! User defined struct export

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    flags::EStructFlags,
    reader::{ArchiveReader, ArchiveWriter},
    types::PackageIndexTrait,
    unversioned::{header::UnversionedHeader, Ancestry},
    Error, FNameContainer,
};
use unreal_asset_properties::Property;

use crate::{BaseExport, NormalExport, StructExport};
use crate::{ExportBaseTrait, ExportNormalTrait, ExportTrait};

/// Struct export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserDefinedStructExport<Index: PackageIndexTrait> {
    /// Base struct export
    pub struct_export: StructExport<Index>,
    /// Struct flags
    #[container_ignore]
    pub flags: EStructFlags,
    /// Default values for the struct
    pub default_struct_instance: Vec<Property>,
}

impl<Index: PackageIndexTrait> UserDefinedStructExport<Index> {
    /// Read a `UserDefinedStructExport` from an asset
    pub fn from_base<Reader: ArchiveReader<Index>>(
        base: &BaseExport<Index>,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let struct_export = StructExport::from_base(base, asset)?;
        let flags = EStructFlags::from_bits(asset.read_u32::<LE>()?)
            .ok_or_else(|| Error::invalid_file("Invalid struct flags".to_string()))?;
        let mut default_struct_instance = Vec::new();
        let mut unversioned_header = UnversionedHeader::new(asset)?;
        let ancestry = Ancestry::new(base.get_class_type_for_ancestry(asset));
        while let Some(e) =
            Property::new(asset, ancestry.clone(), unversioned_header.as_mut(), true)?
        {
            default_struct_instance.push(e);
        }

        Ok(Self {
            struct_export,
            flags,
            default_struct_instance,
        })
    }
}

impl<Index: PackageIndexTrait> ExportNormalTrait<Index> for UserDefinedStructExport<Index> {
    fn get_normal_export(&'_ self) -> Option<&'_ NormalExport<Index>> {
        Some(&self.struct_export.normal_export)
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut NormalExport<Index>> {
        Some(&mut self.struct_export.normal_export)
    }
}

impl<Index: PackageIndexTrait> ExportBaseTrait<Index> for UserDefinedStructExport<Index> {
    fn get_base_export(&'_ self) -> &'_ BaseExport<Index> {
        &self.struct_export.normal_export.base_export
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport<Index> {
        &mut self.struct_export.normal_export.base_export
    }
}

impl<Index: PackageIndexTrait> ExportTrait<Index> for UserDefinedStructExport<Index> {
    fn write<Writer: ArchiveWriter<Index>>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.struct_export.write(asset)?;
        asset.write_u32::<LE>(self.flags.bits())?;
        for entry in &self.default_struct_instance {
            Property::write(entry, asset, true)?;
        }
        let stub = asset.add_fname("None");
        asset.write_fname(&stub)?;
        Ok(())
    }
}
