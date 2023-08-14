//! User defined struct export

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    flags::EStructFlags,
    reader::{ArchiveReader, ArchiveWriter},
    unversioned::{header::UnversionedHeader, Ancestry},
    Error, FNameContainer,
};
use unreal_asset_properties::Property;

use crate::{BaseExport, NormalExport, StructExport};
use crate::{ExportBaseTrait, ExportNormalTrait, ExportTrait};

/// Struct export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserDefinedStructExport {
    /// Base struct export
    pub struct_export: StructExport,
    /// Struct flags
    #[container_ignore]
    pub flags: EStructFlags,
    /// Default values for the struct
    pub default_struct_instance: Vec<Property>,
}

impl UserDefinedStructExport {
    /// Read a `UserDefinedStructExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
        base: &BaseExport,
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

impl ExportNormalTrait for UserDefinedStructExport {
    fn get_normal_export(&'_ self) -> Option<&'_ NormalExport> {
        Some(&self.struct_export.normal_export)
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut NormalExport> {
        Some(&mut self.struct_export.normal_export)
    }
}

impl ExportBaseTrait for UserDefinedStructExport {
    fn get_base_export(&'_ self) -> &'_ BaseExport {
        &self.struct_export.normal_export.base_export
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport {
        &mut self.struct_export.normal_export.base_export
    }
}

impl ExportTrait for UserDefinedStructExport {
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
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
