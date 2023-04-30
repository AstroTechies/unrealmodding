//! Normal export

use crate::error::Error;
use crate::exports::{base_export::BaseExport, ExportBaseTrait, ExportNormalTrait, ExportTrait};
use crate::properties::Property;
use crate::reader::{archive_reader::ArchiveReader, archive_writer::ArchiveWriter};
use crate::unversioned::ancestry::Ancestry;
use crate::unversioned::header::UnversionedHeader;

/// Normal export
///
/// This export is usually the base export for all other exports
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalExport {
    /// Base export
    pub base_export: BaseExport,
    /// Extra data
    pub extras: Vec<u8>,
    /// Properties
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
    /// Read a `NormalExport` from an asset
    pub fn from_base<Reader: ArchiveReader>(
        base: &BaseExport,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let mut properties = Vec::new();

        let mut unversioned_header = UnversionedHeader::new(asset)?;
        let ancestry = Ancestry::new(base.get_class_type_for_ancestry(asset));
        while let Some(e) =
            Property::new(asset, ancestry.clone(), unversioned_header.as_mut(), true)?
        {
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
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error> {
        let (unversioned_header, sorted_properties) = match asset.generate_unversioned_header(
            &self.properties,
            &self.base_export.get_class_type_for_ancestry(asset),
        )? {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        };

        if let Some(unversioned_header) = unversioned_header {
            unversioned_header.write(asset)?;
        }

        let properties = sorted_properties.as_ref().unwrap_or(&self.properties);

        for entry in properties.iter() {
            Property::write(entry, asset, true)?;
        }
        if !asset.has_unversioned_properties() {
            let none = asset.get_name_map().get_mut().add_fname("None");
            asset.write_fname(&none)?;
        }

        Ok(())
    }
}
