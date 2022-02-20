use crate::uasset::unreal_types::{FName, Guid, NamespacedString};


#[derive(Debug, Default)]
pub struct LevelExport {
    class_index: i32,
    super_index: i32,
    template_index: i32,
    outer_index: i32,
    object_name: FName,
    object_flags: u32,
    serial_size: i64,
    serial_offset: i64,
    forced_export: bool,
    not_for_client: bool,
    not_for_server: bool,
    package_guid: Guid,
    package_flags: u32,
    not_always_loaded_for_editor_game: bool,
    is_asset: bool,
    first_export_dependency: i32,
    serialization_before_serialization_dependencies : i32,
    create_before_serialization_dependencies: i32,
    serialization_before_create_dependencies: i32,
    create_before_create_dependencies: i32,

    index_data: Vec<i32>,
    level_type: NamespacedString,
    flags_probably: u64,
    misc_category_data: Vec<i32>
}

