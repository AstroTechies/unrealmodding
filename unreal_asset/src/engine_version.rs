use lazy_static::lazy_static;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    custom_version::CustomVersion,
    object_version::{ObjectVersion, ObjectVersionUE5},
};

/// An enum used to represent all retail versions of the Unreal Engine. Each version entry both a particular ['ObjectVersion'] and the default set of all applicable ['CustomVersion'] enum values.
///
/// ['ObjectVersion']: object_version.html
/// ['CustomVersion']: custom_version.html
#[derive(
    Debug, Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive,
)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum EngineVersion {
    UNKNOWN,
    VER_UE4_OLDEST_LOADABLE_PACKAGE,

    /// 4.0
    VER_UE4_0,
    /// 4.1
    VER_UE4_1,
    /// 4.2
    VER_UE4_2,
    /// 4.3
    VER_UE4_3,
    /// 4.4
    VER_UE4_4,
    /// 4.5
    VER_UE4_5,
    /// 4.6
    VER_UE4_6,
    /// 4.7
    VER_UE4_7,
    /// 4.8
    VER_UE4_8,
    /// 4.9
    VER_UE4_9,
    /// 4.10
    VER_UE4_10,
    /// 4.11
    VER_UE4_11,
    /// 4.12
    VER_UE4_12,
    /// 4.13
    VER_UE4_13,
    /// 4.14
    VER_UE4_14,
    /// 4.15
    VER_UE4_15,
    /// 4.16
    VER_UE4_16,
    /// 4.17
    VER_UE4_17,
    /// 4.18
    VER_UE4_18,
    /// 4.19
    VER_UE4_19,
    /// 4.20
    VER_UE4_20,
    /// 4.21
    VER_UE4_21,
    /// 4.22
    VER_UE4_22,
    /// 4.23
    VER_UE4_23,
    /// 4.24
    VER_UE4_24,
    /// 4.25
    VER_UE4_25,
    /// 4.26
    VER_UE4_26,
    /// 4.27
    VER_UE4_27,

    /// 5.0
    VER_UE5_0,

    /// The newest specified version of the Unreal Engine.
    VER_UE4_AUTOMATIC_VERSION,
    VER_UE4_AUTOMATIC_VERSION_PLUS_ONE,
}

lazy_static! {
    static ref OBJECT_VERSION_TO_ENGINE_VERSION: Vec<(ObjectVersion, EngineVersion)> = Vec::from([
        (
            ObjectVersion::VER_UE4_PRIVATE_REMOTE_ROLE,
            EngineVersion::VER_UE4_0
        ),
        (
            ObjectVersion::VER_UE4_UNDO_BREAK_MATERIALATTRIBUTES_CHANGE,
            EngineVersion::VER_UE4_1
        ),
        (
            ObjectVersion::VER_UE4_FIX_MATERIAL_COORDS,
            EngineVersion::VER_UE4_2
        ),
        (
            ObjectVersion::VER_UE4_FIX_MATERIAL_PROPERTY_OVERRIDE_SERIALIZE,
            EngineVersion::VER_UE4_3
        ),
        (
            ObjectVersion::VER_UE4_BLUEPRINT_USE_SCS_ROOTCOMPONENT_SCALE,
            EngineVersion::VER_UE4_4
        ),
        (
            ObjectVersion::VER_UE4_RENAME_CAMERA_COMPONENT_CONTROL_ROTATION,
            EngineVersion::VER_UE4_5
        ),
        (
            ObjectVersion::VER_UE4_MOVEMENTCOMPONENT_AXIS_SETTINGS,
            EngineVersion::VER_UE4_6
        ),
        (
            ObjectVersion::VER_UE4_AFTER_MERGING_ADD_MODIFIERS_RUNTIME_GENERATION_TO_4_7,
            EngineVersion::VER_UE4_7
        ),
        (
            ObjectVersion::VER_UE4_SERIALIZE_BLUEPRINT_EVENTGRAPH_FASTCALLS_IN_UFUNCTION,
            EngineVersion::VER_UE4_8
        ),
        (
            ObjectVersion::VER_UE4_APEX_CLOTH_TESSELLATION,
            EngineVersion::VER_UE4_9
        ),
        (
            ObjectVersion::VER_UE4_APEX_CLOTH_TESSELLATION,
            EngineVersion::VER_UE4_10
        ),
        (
            ObjectVersion::VER_UE4_STREAMABLE_TEXTURE_MIN_MAX_DISTANCE,
            EngineVersion::VER_UE4_11
        ),
        (
            ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED,
            EngineVersion::VER_UE4_12
        ),
        (
            ObjectVersion::VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR,
            EngineVersion::VER_UE4_13
        ),
        (
            ObjectVersion::VER_UE4_TemplateIndex_IN_COOKED_EXPORTS,
            EngineVersion::VER_UE4_14
        ),
        (
            ObjectVersion::VER_UE4_ADDED_SEARCHABLE_NAMES,
            EngineVersion::VER_UE4_15
        ),
        (
            ObjectVersion::VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG,
            EngineVersion::VER_UE4_16
        ),
        (
            ObjectVersion::VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG,
            EngineVersion::VER_UE4_17
        ),
        (
            ObjectVersion::VER_UE4_ADDED_SOFT_OBJECT_PATH,
            EngineVersion::VER_UE4_18
        ),
        (
            ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID,
            EngineVersion::VER_UE4_19
        ),
        (
            ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID,
            EngineVersion::VER_UE4_20
        ),
        (
            ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC,
            EngineVersion::VER_UE4_21
        ),
        (
            ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC,
            EngineVersion::VER_UE4_22
        ),
        (
            ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC,
            EngineVersion::VER_UE4_23
        ),
        (
            ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER,
            EngineVersion::VER_UE4_24
        ),
        (
            ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER,
            EngineVersion::VER_UE4_25
        ),
        (
            ObjectVersion::VER_UE4_SKINWEIGHT_PROFILE_DATA_LAYOUT_CHANGES,
            EngineVersion::VER_UE4_26
        ),
        (
            ObjectVersion::VER_UE4_CORRECT_LICENSEE_FLAG,
            EngineVersion::VER_UE4_27
        ),
        (
            ObjectVersion::VER_UE4_CORRECT_LICENSEE_FLAG,
            EngineVersion::VER_UE5_0
        ),
    ]);
    static ref OBJECT_VERSION_TO_ENGINE_VERSION_UE5: Vec<(ObjectVersionUE5, EngineVersion)> =
        Vec::from([(
            ObjectVersionUE5::LARGE_WORLD_COORDINATES,
            EngineVersion::VER_UE4_5
        )]);
}

pub fn get_possible_versions(
    object_version: ObjectVersion,
    object_version_ue5: ObjectVersionUE5,
) -> Vec<EngineVersion> {
    let mut possible_versions = Vec::new();
    if let Some(ue5_version) = OBJECT_VERSION_TO_ENGINE_VERSION_UE5
        .iter()
        .find(|(version, _)| *version == object_version_ue5)
        .map(|(_, engine_version)| *engine_version)
    {
        possible_versions.push(ue5_version);
    }

    possible_versions.extend(
        OBJECT_VERSION_TO_ENGINE_VERSION
            .iter()
            .find(|(version, _)| *version == object_version)
            .map(|(_, engine_version)| *engine_version),
    );

    possible_versions
}

pub fn get_object_versions(engine_version: EngineVersion) -> (ObjectVersion, ObjectVersionUE5) {
    if engine_version == EngineVersion::UNKNOWN {
        return (ObjectVersion::UNKNOWN, ObjectVersionUE5::UNKNOWN);
    }

    let object_version = OBJECT_VERSION_TO_ENGINE_VERSION
        .iter()
        .find(|(_, version)| *version == engine_version)
        .map(|(object_version, _)| *object_version)
        .unwrap_or(ObjectVersion::UNKNOWN);

    let object_version_ue5 = OBJECT_VERSION_TO_ENGINE_VERSION_UE5
        .iter()
        .find(|(_, version)| *version == engine_version)
        .map(|(object_version, _)| *object_version)
        .unwrap_or(ObjectVersionUE5::UNKNOWN);

    (object_version, object_version_ue5)
}

pub fn guess_engine_version(
    object_version: ObjectVersion,
    object_version_ue5: ObjectVersionUE5,
    custom_versions: &[CustomVersion],
) -> EngineVersion {
    let possible_versions = get_possible_versions(object_version, object_version_ue5);

    if possible_versions.is_empty() {
        return EngineVersion::UNKNOWN;
    }

    if possible_versions.len() == 1 {
        return possible_versions[0];
    }

    let mut min_introduced = EngineVersion::VER_UE4_OLDEST_LOADABLE_PACKAGE;
    let mut max_introduced = EngineVersion::VER_UE4_AUTOMATIC_VERSION_PLUS_ONE;

    for custom_version in custom_versions {
        let current_min_introduced = custom_version
            .get_engine_version_from_version_number(custom_version.version)
            .unwrap_or(EngineVersion::UNKNOWN);
        let current_max_introduced = custom_version
            .get_engine_version_from_version_number(custom_version.version + 1)
            .unwrap_or(EngineVersion::UNKNOWN);

        if current_min_introduced != EngineVersion::UNKNOWN
            && current_min_introduced > min_introduced
        {
            min_introduced = current_min_introduced;
        }

        if current_max_introduced != EngineVersion::UNKNOWN
            && current_max_introduced < max_introduced
        {
            max_introduced = current_max_introduced;
        }
    }

    let mut final_possible_versions = possible_versions
        .iter()
        .filter(|e| **e >= min_introduced && **e < max_introduced)
        .map(|e| *e)
        .collect::<Vec<_>>();

    final_possible_versions.sort();

    if final_possible_versions.is_empty() {
        // there must be a special set of custom versions; we'll just ignore our intuitions and go with the object version alone
        return possible_versions[0];
    }

    final_possible_versions
        .first()
        .map(|e| *e)
        .unwrap_or(EngineVersion::UNKNOWN)
}
