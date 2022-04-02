use crate::uasset::unreal_types::{FName, Guid};


#[derive(Debug, Default)]
pub struct UnknownExport {
    pub class_index: i32,
    pub super_index: i32,
    pub template_index: i32,
    pub outer_index: i32,
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
    pub first_export_dependency: i32,
    pub serialization_before_serialization_dependencies : i32,
    pub create_before_serialization_dependencies: i32,
    pub serialization_before_create_dependencies: i32,
    pub create_before_create_dependencies: i32
}