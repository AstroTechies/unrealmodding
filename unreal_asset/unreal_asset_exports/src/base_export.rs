//! Base uasset export

use num_enum::{IntoPrimitive, TryFromPrimitive};

use unreal_asset_base::{
    flags::EObjectFlags,
    reader::{ArchiveTrait, ArchiveWriter},
    types::{FName, PackageIndex},
    Error, FNameContainer, Guid,
};

use crate::{ExportBaseTrait, ExportNormalTrait, ExportTrait};

/// Export filter flags
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EExportFilterFlags {
    /// None
    None,
    /// This export should not be loaded on the client
    NotForClient,
    /// This export should not be loaded on the server
    NotForServer,
}

/// Minimal information about an export
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BaseExport {
    /// Class index
    #[container_ignore]
    pub class_index: PackageIndex,
    /// Zen class index

    /// Super index
    #[container_ignore]
    pub super_index: PackageIndex,
    /// Zen super index

    /// Template index
    #[container_ignore]
    pub template_index: PackageIndex,
    /// Zen template index

    /// Outer index
    #[container_ignore]
    pub outer_index: PackageIndex,
    /// Zen outer index

    /// Object name
    pub object_name: FName,
    /// Object flags
    #[container_ignore]
    pub object_flags: EObjectFlags,
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
    /// Is inherited instance
    pub is_inherited_instance: bool,
    /// Package flags
    pub package_flags: u32,
    /// Is not always loaded for editor game
    pub not_always_loaded_for_editor_game: bool,
    /// Is an asset
    pub is_asset: bool,
    /// Generate public hash
    pub generate_public_hash: bool,
    /// Public export hash. Interpreted as a global import PackageObjectIndex in UE4 assets
    pub public_export_hash: u64,
    /// First dependency serialized offset
    pub first_export_dependency_offset: i32,
    /// Dependencies that should be serialized before this export is serialized
    #[container_ignore]
    pub serialization_before_serialization_dependencies: Vec<PackageIndex>,

    /// Dependencies that should be created before this export is serialized
    #[container_ignore]
    pub create_before_serialization_dependencies: Vec<PackageIndex>,

    /// Dependencies that should be serialized before this export is created
    #[container_ignore]
    pub serialization_before_create_dependencies: Vec<PackageIndex>,

    /// Dependencies that should be created before this export is created
    #[container_ignore]
    pub create_before_create_dependencies: Vec<PackageIndex>,
}

impl BaseExport {
    /// Gets class type for first ancestry parent
    pub fn get_class_type_for_ancestry<Asset: ArchiveTrait>(&self, asset: &Asset) -> FName {
        match self.class_index.is_import() {
            true => asset.get_import(self.class_index).map(|e| e.object_name),
            false => asset.get_parent_class_export_name(),
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
