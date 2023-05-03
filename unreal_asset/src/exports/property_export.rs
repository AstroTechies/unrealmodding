//! Property export

use byteorder::LittleEndian;
use unreal_asset_proc_macro::FNameContainer;

use crate::exports::{
    base_export::BaseExport, normal_export::NormalExport, ExportBaseTrait, ExportNormalTrait,
    ExportTrait,
};
use crate::implement_get;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::uproperty::{UProperty, UPropertyTrait};
use crate::Error;

/// Property export
///
/// This is a `UProperty` export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyExport {
    /// Base normal export
    pub normal_export: NormalExport,
    /// Property
    pub property: UProperty,
}

implement_get!(PropertyExport);

impl PropertyExport {
    /// Read a `PropertyExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
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
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;
        asset.write_i32::<LittleEndian>(0)?;
        self.property.write(asset)?;
        Ok(())
    }
}
