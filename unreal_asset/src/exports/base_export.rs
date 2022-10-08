use crate::error::Error;
use crate::exports::{ExportBaseTrait, ExportNormalTrait, ExportTrait};
use crate::reader::asset_writer::AssetWriter;
use crate::unreal_types::{FName, Guid, PackageIndex};

#[derive(Debug, Default, Clone)]
pub struct BaseExport {
    pub class_index: PackageIndex,
    pub super_index: PackageIndex,
    pub template_index: PackageIndex,
    pub outer_index: PackageIndex,
    pub object_name: FName,
    pub object_flags: u32,
    pub serial_size: i64,
    pub serial_offset: i64,
    pub forced_export: bool,
    pub not_for_client: bool,
    pub not_for_server: bool,
    pub package_guid: Guid,
    pub package_flags: u32,
    pub not_always_loaded_for_editor_game: bool,
    pub is_asset: bool,
    pub first_export_dependency_offset: i32,
    pub serialization_before_serialization_dependencies: Vec<PackageIndex>,
    pub(crate) serialization_before_serialization_dependencies_size: i32,

    pub create_before_serialization_dependencies: Vec<PackageIndex>,
    pub(crate) create_before_serialization_dependencies_size: i32,

    pub serialization_before_create_dependencies: Vec<PackageIndex>,
    pub(crate) serialization_before_create_dependencies_size: i32,

    pub create_before_create_dependencies: Vec<PackageIndex>,
    pub(crate) create_before_create_dependencies_size: i32,
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
    fn write<Writer: AssetWriter>(&self, _asset: &mut Writer) -> Result<(), Error> {
        Ok(())
    }
}
