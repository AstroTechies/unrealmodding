use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExtendedPattern {
    pub pattern: String,
    pub first_op_codes: u32,
    pub total_byte_instruction: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GInfoPatterns {
    pub g_name: Option<ExtendedPattern>,
    pub g_object: Option<ExtendedPattern>,
    pub g_world: Option<ExtendedPattern>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GInfoOffsets {
    pub g_name: Option<u32>,
    pub g_object: Option<u32>,
    pub g_world: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GInfo {
    Patterns(GInfoPatterns),
    Offsets(GInfoOffsets),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UObjectDef {
    pub index: u32,
    pub class: u32,
    pub name: u32,
    pub outer: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UFieldDef {
    pub next: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UStructDef {
    pub super_struct: u32,
    pub children: u32,
    pub properties_size: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UFunctionDef {
    pub function_flags: u32,
    pub func: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FField {
    pub name: u32,
    pub next: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Property {
    pub array_dim: u32,
    pub offset: u32,
    pub fproperty: Option<FField>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionInfoPatterns {
    pub game_state_init: Option<String>,
    pub begin_play: Option<String>,
    pub static_load_object: Option<String>,
    pub spawn_actor_ftrans: Option<String>,
    pub call_function_by_name_with_arguments: Option<String>,
    pub process_event: Option<String>,
    pub create_default_object: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionInfoOffsets {
    pub game_state_init: Option<u32>,
    pub begin_play: Option<u32>,
    pub static_load_object: Option<u32>,
    pub spawn_actor_ftrans: Option<u32>,
    pub call_function_by_name_with_arguments: Option<u32>,
    pub process_event: Option<u32>,
    pub create_default_object: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FunctionInfo {
    Patterns(FunctionInfoPatterns),
    Offsets(FunctionInfoOffsets),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessInternalFunction {
    pub process_internal: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct StaticConstructObject_InternalInfo {
    pub is_using_updated_static_construct: bool,
    pub static_construct_object_internal_function: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameSettings {
    pub is_using_fchunked_fixed_uobject_array: bool,
    pub uses_fname_pool: bool,
    pub is_using_deferred_spawn: bool,
    pub is_using_4_22: bool,
    pub is_default_object_arrayed: bool,
    pub delay_gui_spawn: bool,

    pub begin_play_overwrite: Option<String>,
    pub g_info_settings: Option<GInfo>,
    pub uobject_def_settings: Option<UObjectDef>,
    pub ufield_def_settings: Option<UFieldDef>,
    pub ustruct_def_settings: Option<UStructDef>,
    pub ufunction_def_settings: Option<UFunctionDef>,
    pub property_settings: Option<Property>,
    pub function_info_settings: Option<FunctionInfo>,
    pub process_internal_function_settings: Option<ProcessInternalFunction>,
    pub static_construct_object_internal_info_settings: Option<StaticConstructObject_InternalInfo>,
}
