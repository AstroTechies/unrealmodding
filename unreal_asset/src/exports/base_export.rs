//! Base uasset export

use crate::error::Error;
use crate::exports::{ExportBaseTrait, ExportNormalTrait, ExportTrait};
use crate::reader::archive_trait::ArchiveTrait;
use crate::reader::archive_writer::ArchiveWriter;
use crate::types::{FName, Guid, PackageIndex};

/// Minimal information about an export
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct BaseExport {
    /// Class index
    pub class_index: PackageIndex,
    /// Super index
    pub super_index: PackageIndex,
    /// Template index
    pub template_index: PackageIndex,
    /// Outer index
    pub outer_index: PackageIndex,
    /// Object name
    pub object_name: FName,
    /// Object flags
    pub object_flags: u32,
    /// Serialized size
    pub serial_size: i64,
    /// Serialized offset
    pub serial_offset: i64,
    /// Is forced export
    pub forced_export: bool,
    /// Is not for client
    pub not_for_client: bool,
    /// Is not for server
    pub not_for_server: bool,
    /// Package guid
    pub package_guid: Guid,
    /// Package flags
    pub package_flags: u32,
    /// Is not always loaded for editor game
    pub not_always_loaded_for_editor_game: bool,
    /// Is an asset
    pub is_asset: bool,
    /// First dependency serialized offset
    pub first_export_dependency_offset: i32,
    /// Dependencies that should be serialized before this export is serialized
    pub serialization_before_serialization_dependencies: Vec<PackageIndex>,
    pub(crate) serialization_before_serialization_dependencies_size: i32,

    /// Dependencies that should be created before this export is serialized
    pub create_before_serialization_dependencies: Vec<PackageIndex>,
    pub(crate) create_before_serialization_dependencies_size: i32,

    /// Dependencies that should be serialized before this export is created
    pub serialization_before_create_dependencies: Vec<PackageIndex>,
    pub(crate) serialization_before_create_dependencies_size: i32,

    /// Dependencies that should be created before this export is created
    pub create_before_create_dependencies: Vec<PackageIndex>,
    pub(crate) create_before_create_dependencies_size: i32,
}

impl BaseExport {
    /// Gets class type for first ancestry parent
    pub fn get_class_type_for_ancestry<Asset: ArchiveTrait>(&self, asset: &Asset) -> FName {
        match self.class_index.is_import() {
            true => asset
                .get_import(self.class_index)
                .map(|e| e.object_name.clone()),
            false => asset.get_parent_class().map(|e| e.parent_class_export_name),
        }
        .unwrap_or_default()
    }
}

impl ExportNormalTrait for BaseExport {
    fn get_normal_export(&'_ self) -> Option<&'_ super::normal_export::NormalExport> {
        None
    }

    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut super::normal_export::NormalExport> {
        None
    }
}

impl ExportBaseTrait for BaseExport {
    fn get_base_export(&'_ self) -> &'_ BaseExport {
        self
    }

    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport {
        self
    }
}

impl ExportTrait for BaseExport {
    fn write<Writer: ArchiveWriter>(&self, _asset: &mut Writer) -> Result<(), Error> {
        Ok(())
    }
}
