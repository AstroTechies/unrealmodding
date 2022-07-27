use std::{collections::HashMap, fmt::Display};

use crate::{
    error::Error,
    reader::{asset_reader::AssetReader, asset_writer::AssetWriter},
};

use super::{
    ue4version,
    unreal_types::{new_guid, Guid},
};
use byteorder::LittleEndian;
use lazy_static::lazy_static;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct CustomVersion {
    pub guid: Guid,
    pub friendly_name: Option<String>,
    pub version: i32,
}

#[rustfmt::skip]
lazy_static! {
    static ref GUID_TO_FRIENDLY_NAME: HashMap<Guid, String> = HashMap::from([
        ( new_guid(0, 0, 0, 0xF99D40C1), String::from("UnusedCustomVersionKey") ),
        ( new_guid(0xB0D832E4, 0x1F894F0D, 0xACCF7EB7, 0x36FD4AA2), String::from("FBlueprintsObjectVersion") ),
        ( new_guid(0xE1C64328, 0xA22C4D53, 0xA36C8E86, 0x6417BD8C), String::from("FBuildObjectVersion") ),
        ( new_guid(0xB02B49B5, 0xBB2044E9, 0xA30432B7, 0x52E40360), String::from("FMobileObjectVersion") ),
        ( new_guid(0xA4E4105C, 0x59A149B5, 0xA7C540C4, 0x547EDFEE), String::from("FNetworkingObjectVersion") ),
        ( new_guid(0x39C831C9, 0x5AE647DC, 0x9A449C17, 0x3E1C8E7C), String::from("FOnlineObjectVersion") ),
        ( new_guid(0x78F01B33, 0xEBEA4F98, 0xB9B484EA, 0xCCB95AA2), String::from("FPhysicsObjectVersion") ),
        ( new_guid(0x6631380F, 0x2D4D43E0, 0x8009CF27, 0x6956A95A), String::from("FPlatformObjectVersion") ),
        ( new_guid(0x12F88B9F, 0x88754AFC, 0xA67CD90C, 0x383ABD29), String::from("FRenderingObjectVersion") ),
        ( new_guid(0x7B5AE74C, 0xD2704C10, 0xA9585798, 0x0B212A5A), String::from("FSequencerObjectVersion") ),
        ( new_guid(0xD7296918, 0x1DD64BDD, 0x9DE264A8, 0x3CC13884), String::from("FVRObjectVersion") ),
        ( new_guid(0xC2A15278, 0xBFE74AFE, 0x6C1790FF, 0x531DF755), String::from("FLoadTimesObjectVersion") ),
        ( new_guid(0x6EACA3D4, 0x40EC4CC1, 0xB7868BED, 0x9428FC5), String::from("FGeometryObjectVersion") ),
        ( new_guid(0x29E575DD, 0xE0A34627, 0x9D10D276, 0x232CDCEA), String::from("FAnimPhysObjectVersion") ), //
        ( new_guid(0xAF43A65D, 0x7FD34947, 0x98733E8E, 0xD9C1BB05), String::from("FAnimObjectVersion") ),
        ( new_guid(0x6B266CEC, 0x1EC74B8F, 0xA30BE4D9, 0x0942FC07), String::from("FReflectionCaptureObjectVersion") ),
        ( new_guid(0x0DF73D61, 0xA23F47EA, 0xB72789E9, 0x0C41499A), String::from("FAutomationObjectVersion") ),
        ( new_guid(0x9DFFBCD6, 0x494F0158, 0xE2211282, 0x3C92A888), String::from("FEnterpriseObjectVersion") ),
        ( new_guid(0xF2AED0AC, 0x9AFE416F, 0x8664AA7F, 0xFA26D6FC), String::from("FNiagaraObjectVersion") ),
        ( new_guid(0x174F1F0B, 0xB4C645A5, 0xB13F2EE8, 0xD0FB917D), String::from("FDestructionObjectVersion") ),
        ( new_guid(0x35F94A83, 0xE258406C, 0xA31809F5, 0x9610247C), String::from("FExternalPhysicsCustomObjectVersion") ),
        ( new_guid(0xB68FC16E, 0x8B1B42E2, 0xB453215C, 0x058844FE), String::from("FExternalPhysicsMaterialCustomObjectVersion") ),
        ( new_guid(0xB2E18506, 0x4273CFC2, 0xA54EF4BB, 0x758BBA07), String::from("FCineCameraObjectVersion") ),
        ( new_guid(0x64F58936, 0xFD1B42BA, 0xBA967289, 0xD5D0FA4E), String::from("FVirtualProductionObjectVersion") ),
        ( new_guid(0x6f0ed827, 0xa6094895, 0x9c91998d, 0x90180ea4), String::from("FMediaFrameworkObjectVersion") ),
        ( new_guid(0xAFE08691, 0x3A0D4952, 0xB673673B, 0x7CF22D1E), String::from("FPoseDriverCustomVersion") ),
        ( new_guid(0xCB8AB0CD, 0xE78C4BDE, 0xA8621393, 0x14E9EF62), String::from("FTempCustomVersion") ),
        ( new_guid(0x2EB5FDBD, 0x01AC4D10, 0x8136F38F, 0x3393A5DA), String::from("FAnimationCustomVersion") ),
        ( new_guid(0x717F9EE7, 0xE9B0493A, 0x88B39132, 0x1B388107), String::from("FAssetRegistryVersion") ),
        ( new_guid(0xFB680AF2, 0x59EF4BA3, 0xBAA819B5, 0x73C8443D), String::from("FClothingAssetCustomVersion") ),
        ( new_guid(0x4A56EB40, 0x10F511DC, 0x92D3347E, 0xB2C96AE7), String::from("FParticleSystemCustomVersion") ),
        ( new_guid(0xD78A4A00, 0xE8584697, 0xBAA819B5, 0x487D46B4), String::from("FSkeletalMeshCustomVersion") ),
        ( new_guid(0x5579F886, 0x933A4C1F, 0x83BA087B, 0x6361B92F), String::from("FRecomputeTangentCustomVersion") ),
        ( new_guid(0x612FBE52, 0xDA53400B, 0x910D4F91, 0x9FB1857C), String::from("FOverlappingVerticesCustomVersion") ),
        ( new_guid(0x430C4D19, 0x71544970, 0x87699B69, 0xDF90B0E5), String::from("FFoliageCustomVersion") ),
        ( new_guid(0xaafe32bd, 0x53954c14, 0xb66a5e25, 0x1032d1dd), String::from("FProceduralFoliageCustomVersion") ),
        ( new_guid(0xab965196, 0x45d808fc, 0xb7d7228d, 0x78ad569e), String::from("FLiveLinkCustomVersion") ),

        ( FCoreObjectVersion::guid(), String::from(FCoreObjectVersion::friendly_name()) ),
        ( FEditorObjectVersion::guid(), String::from(FEditorObjectVersion::friendly_name()) ),
        ( FFrameworkObjectVersion::guid(), String::from(FFrameworkObjectVersion::friendly_name()) ),
        ( FFortniteMainBranchObjectVersion::guid(), String::from(FFortniteMainBranchObjectVersion::friendly_name()) ),
        ( FReleaseObjectVersion::guid(), String::from(FReleaseObjectVersion::friendly_name()) )
    ]);
}

impl CustomVersion {
    pub fn new(guid: Guid, version: i32) -> Self {
        let friendly_name = GUID_TO_FRIENDLY_NAME.get(&guid).map(|e| e.to_owned());
        CustomVersion {
            guid,
            friendly_name,
            version,
        }
    }

    pub fn read<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let mut key = [0u8; 16];
        asset.read_exact(&mut key)?;
        let version = asset.read_i32::<LittleEndian>()?;
        Ok(Self {
            guid: key,
            friendly_name: None,
            version,
        })
    }

    pub fn write<Writer: AssetWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        writer.write_all(&self.guid)?;
        writer.write_i32::<LittleEndian>(self.version)?;
        Ok(())
    }

    pub fn from_version<T>(version: T) -> Self
    where
        T: CustomVersionTrait + Into<i32>,
    {
        CustomVersion {
            guid: T::guid(),
            friendly_name: Some(String::from(T::friendly_name())),
            version: version.into(),
        }
    }
}

pub trait CustomVersionTrait {
    fn from_engine_version(version: i32) -> Self;
    fn friendly_name() -> &'static str;
    fn guid() -> Guid;
}

macro_rules! impl_custom_version_trait {
    ($enum_name:ident, $friendly_name:expr, $guid:expr, $($ver_name:ident : $ver:ident),*) => {
        impl CustomVersionTrait for $enum_name {
            fn from_engine_version(version: i32) -> Self {
                $(
                    if version >= ue4version::$ver_name {
                        return $enum_name::$ver;
                    }
                )*

                return $enum_name::BeforeCustomVersionWasAdded;
            }

            fn friendly_name() -> &'static str {
                $friendly_name
            }

            fn guid() -> Guid {
                $guid
            }
        }
    }
}

#[derive(IntoPrimitive)]
#[repr(i32)]
pub enum FFortniteMainBranchObjectVersion {
    // Before any version changes were made
    // Introduced: UE4Version.VER_UE4_OLDEST_LOADABLE_PACKAGE
    BeforeCustomVersionWasAdded = 0,

    // World composition tile offset changed from 2d to 3d
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    WorldCompositionTile3DOffset,

    // Minor material serialization optimization
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    MaterialInstanceSerializeOptimizationShaderFname,

    // Refactored cull distances to account for HLOD, explicit override and globals in priority
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    CullDistanceRefactorRemovedDefaultDistance,
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    CullDistanceRefactorNeverCullHlodsByDefault,
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    CullDistanceRefactorNeverCullAlodactorsByDefault,

    // Support to remove morphtarget generated by bRemapMorphtarget
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    SaveGeneratedMorphTargetByEngine,

    // Convert reduction setting options
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    ConvertReductionSettingOptions,

    // Serialize the type of blending used for landscape layer weight static params
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    StaticParameterTerrainLayerWeightBlendType,

    // Fix up None Named animation curve names,
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    FixUpNoneNameAnimationCurves,

    // Ensure ActiveBoneIndices to have parents even not skinned for old assets
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    EnsureActiveBoneIndicesToContainParents,

    // Serialize the instanced static mesh render data, to avoid building it at runtime
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    SerializeInstancedStaticMeshRenderData,

    // Cache material quality node usage
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    CachedMaterialQualityNodeUsage,

    // Font outlines no longer apply to drop shadows for new objects but we maintain the opposite way for backwards compat
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    FontOutlineDropShadowFixup,

    // New skeletal mesh import workflow (Geometry only or animation only re-import )
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    NewSkeletalMeshImporterWorkflow,

    // Migrate data from previous data structure to new one to support materials per LOD on the Landscape
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    NewLandscapeMaterialPerLOD,

    // New Pose Asset data type
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    RemoveUnnecessaryTracksFromPose,

    // Migrate Foliage TLazyObjectPtr to TSoftObjectPtr
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    FoliageLazyObjPtrToSoftObjPtr,

    // TimelineTemplates store their derived names instead of dynamically generating. This code tied to this version was reverted and redone at a later date
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    RevertedStoreTimelineNamesInTemplate,

    // Added BakePoseOverride for LOD setting
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    AddBakePoseOverrideForSkeletalMeshReductionSetting,

    // TimelineTemplates store their derived names instead of dynamically generating
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    StoreTimelineNamesInTemplate,

    // New Pose Asset data type
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    WidgetStopDuplicatingAnimations,

    // Allow reducing of the base LOD, we need to store some imported model data so we can reduce again from the same data.
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    AllowSkeletalMeshToReduceTheBaseLOD,

    // Curve Table size reduction
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    ShrinkCurveTableSize,

    // Widgets upgraded with WidgetStopDuplicatingAnimations, may not correctly default-to-self for the widget parameter.
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    WidgetAnimationDefaultToSelfFail,

    // HUDWidgets now require an element tag
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    FortHUDElementNowRequiresTag,

    // Animation saved as bulk data when cooked
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    FortMappedCookedAnimation,

    // Support Virtual Bone in Retarget Manager
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    SupportVirtualBoneInRetargeting,

    // Fixup bad defaults in water metadata
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    FixUpWaterMetadata,

    // Move the location of water metadata
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    MoveWaterMetadataToActor,

    // Replaced lake collision component
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    ReplaceLakeCollision,

    // Anim layer node names are now conformed by Guid
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    AnimLayerGuidConformation,

    // Ocean collision component has become dynamic
    // Introduced: UE4Version.VER_UE4_SKINWEIGHT_PROFILE_DATA_LAYOUT_CHANGES
    MakeOceanCollisionTransient,

    // FFieldPath will serialize the owner struct reference and only a short path to its property
    // Introduced: UE4Version.VER_UE4_SKINWEIGHT_PROFILE_DATA_LAYOUT_CHANGES
    FFieldPathOwnerSerialization,

    // Simplified WaterBody post process material handling
    // Introduced: UE4Version.VER_UE4_SKINWEIGHT_PROFILE_DATA_LAYOUT_CHANGES
    FixUpUnderwaterPostProcessMaterial,

    // A single water exclusion volume can now exclude N water bodies
    // Introduced: UE4Version.VER_UE4_SKINWEIGHT_PROFILE_DATA_LAYOUT_CHANGES
    SupportMultipleWaterBodiesPerExclusionVolume,

    // Serialize rigvm operators one by one instead of the full byte code array to ensure determinism
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    RigVMByteCodeDeterminism,

    // Serialize the physical materials generated by the render material
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    LandscapePhysicalMaterialRenderData,

    // RuntimeVirtualTextureVolume fix transforms
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    FixupRuntimeVirtualTextureVolume,

    // Retrieve water body collision components that were lost in cooked builds
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    FixUpRiverCollisionComponents,

    // Fix duplicate spline mesh components on rivers
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    FixDuplicateRiverSplineMeshCollisionComponents,

    // Indicates level has stable actor guids
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    ContainsStableActorGUIDs,

    // Levelset Serialization support for BodySetup.
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    LevelsetSerializationSupportForBodySetup,

    // Moving Chaos solver properties to allow them to exist in the project physics settings
    // Introduced: UE4Version.VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS
    ChaosSolverPropertiesMoved,

    // Moving some UFortGameFeatureData properties and behaviors into the UGameFeatureAction pattern
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    GameFeatureDataMovedComponentListAndCheats,

    // Add centrifugal forces for cloth
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    ChaosClothAddfictitiousforces,

    // Chaos Convex StructureData supports different index sizes based on num verts/planes. Chaos FConvex uses array of FVec3s for vertices instead of particles (Merged from //UE4/Main)
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    ChaosConvexVariableStructureDataAndVerticesArray,

    // Remove the WaterVelocityHeightTexture dependency on MPC_Landscape and LandscapeWaterIndo
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    RemoveLandscapeWaterInfo,

    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION_PLUS_ONE
    VersionPlusOne,
    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION
    LatestVersion = (FFortniteMainBranchObjectVersion::VersionPlusOne as i32) + 1,
}

impl_custom_version_trait!(
    FFortniteMainBranchObjectVersion,
    "FFortniteMainBranchObjectVersion",
    new_guid(0x601D1886, 0xAC644F84, 0xAA16D3DE, 0x0DEAC7D6),
    VER_UE4_AUTOMATIC_VERSION: LatestVersion,
    VER_UE4_AUTOMATIC_VERSION_PLUS_ONE: VersionPlusOne,
    VER_UE4_CORRECT_LICENSEE_FLAG: RemoveLandscapeWaterInfo,
    VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS: ChaosSolverPropertiesMoved,
    VER_UE4_NON_OUTER_PACKAGE_IMPORT: LevelsetSerializationSupportForBodySetup,
    VER_UE4_SKINWEIGHT_PROFILE_DATA_LAYOUT_CHANGES: SupportMultipleWaterBodiesPerExclusionVolume,
    VER_UE4_ADDED_PACKAGE_OWNER: AnimLayerGuidConformation,
    VER_UE4_FIX_WIDE_STRING_CRC: ReplaceLakeCollision,
    VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID: FoliageLazyObjPtrToSoftObjPtr,
    VER_UE4_OLDEST_LOADABLE_PACKAGE: BeforeCustomVersionWasAdded
);

//
// Custom serialization version for changes made in Dev-Framework stream.

#[derive(IntoPrimitive)]
#[repr(i32)]
pub enum FFrameworkObjectVersion {
    // Before any version changes were made
    // Introduced: UE4Version.VER_UE4_OLDEST_LOADABLE_PACKAGE
    BeforeCustomVersionWasAdded = 0,

    // BodySetup's default instance collision profile is used by default when creating a new instance.
    // Introduced: UE4Version.VER_UE4_STREAMABLE_TEXTURE_AABB
    UseBodySetupCollisionProfile,

    // Regenerate subgraph arrays correctly in animation blueprints to remove duplicates and add missing graphs that appear read only when edited
    // Introduced: UE4Version.VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG
    AnimBlueprintSubgraphFix,

    // Static and skeletal mesh sockets now use the specified scale
    // Introduced: UE4Version.VER_UE4_NAME_HASHES_SERIALIZED
    MeshSocketScaleUtilization,

    // Attachment rules are now explicit in how they affect location, rotation and scale
    // Introduced: UE4Version.VER_UE4_NAME_HASHES_SERIALIZED
    ExplicitAttachmentRules,

    // Moved compressed anim data from uasset to the DDC
    // Introduced: UE4Version.VER_UE4_NAME_HASHES_SERIALIZED
    MoveCompressedAnimDataToTheDDC,

    // Some graph pins created using legacy code seem to have lost the RF_Transactional flag, which causes issues with undo. Restore the flag at this version
    // Introduced: UE4Version.VER_UE4_NAME_HASHES_SERIALIZED
    FixNonTransactionalPins,

    // Create new struct for SmartName, and use that for CurveName
    // Introduced: UE4Version.VER_UE4_NAME_HASHES_SERIALIZED
    SmartNameRefactor,

    // Add Reference Skeleton to Rig
    // Introduced: UE4Version.VER_UE4_NAME_HASHES_SERIALIZED
    AddSourceReferenceSkeletonToRig,

    // Refactor ConstraintInstance so that we have an easy way to swap behavior paramters
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    ConstraintInstanceBehaviorParameters,

    // Pose Asset support mask per bone
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    PoseAssetSupportPerBoneMask,

    // Physics Assets now use SkeletalBodySetup instead of BodySetup
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    PhysAssetUseSkeletalBodySetup,

    // Remove SoundWave CompressionName
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    RemoveSoundWaveCompressionName,

    // Switched render data for clothing over to unreal data, reskinned to the simulation mesh
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    AddInternalClothingGraphicalSkinning,

    // Wheel force offset is now applied at the wheel instead of vehicle COM
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    WheelOffsetIsFromWheel,

    // Move curve metadata to be saved in skeleton. Individual asset still saves some flag - i.e. disabled curve and editable or not, but major flag - i.e. material types - moves to skeleton and handle in one place
    // Introduced: UE4Version.VER_UE4_COMPRESSED_SHADER_RESOURCES
    MoveCurveTypesToSkeleton,

    // Cache destructible overlaps on save
    // Introduced: UE4Version.VER_UE4_TemplateIndex_IN_COOKED_EXPORTS
    CacheDestructibleOverlaps,

    // Added serialization of materials applied to geometry cache objects
    // Introduced: UE4Version.VER_UE4_TemplateIndex_IN_COOKED_EXPORTS
    GeometryCacheMissingMaterials,

    // Switch static and skeletal meshes to calculate LODs based on resolution-independent screen size
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    LODsUseResolutionIndependentScreenSize,

    // Blend space post load verification
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    BlendSpacePostLoadSnapToGrid,

    // Addition of rate scales to blend space samples
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    SupportBlendSpaceRateScale,

    // LOD hysteresis also needs conversion from the LODsUseResolutionIndependentScreenSize version
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    LODHysteresisUseResolutionIndependentScreenSize,

    // AudioComponent override subtitle priority default change
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    ChangeAudioComponentOverrideSubtitlePriorityDefault,

    // Serialize hard references to sound files when possible
    // Introduced: UE4Version.VER_UE4_64BIT_EXPORTMAP_SERIALSIZES
    HardSoundReferences,

    // Enforce const correctness in Animation Blueprint function graphs
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    EnforceConstInAnimBlueprintFunctionGraphs,

    // Upgrade the InputKeySelector to use a text style
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    InputKeySelectorTextStyle,

    // Represent a pins container type as an enum not 3 independent booleans
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    EdGraphPinContainerType,

    // Switch asset pins to store as string instead of hard object reference
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    ChangeAssetPinsToString,

    // Fix Local Variables so that the properties are correctly flagged as blueprint visible
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    LocalVariablesBlueprintVisible,

    // Stopped serializing UField_Next so that UFunctions could be serialized in dependently of a UClass in order to allow us to do all UFunction loading in a single pass (after classes and CDOs are created)
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    RemoveUfieldNext,

    // Fix User Defined structs so that all members are correct flagged blueprint visible
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    UserDefinedStructsBlueprintVisible,

    // FMaterialInput and FEdGraphPin store their name as FName instead of FString
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    PinsStoreFName,

    // User defined structs store their default instance, which is used for initializing instances
    // Introduced: UE4Version.VER_UE4_POINTLIGHT_SOURCE_ORIENTATION
    UserDefinedStructsStoreDefaultInstance,

    // Function terminator nodes serialize an FMemberReference rather than a name/class pair
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    FunctionTerminatorNodesUseMemberReference,

    // Custom event and non-native interface event implementations add 'const' to reference parameters
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    EditableEventsUseConstRefParameters,

    // No longer serialize the legacy flag that indicates this state, as it is now implied since we don't serialize the skeleton CDO
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    BlueprintGeneratedClassIsAlwaysAuthoritative,

    // Enforce visibility of blueprint functions - e.g. raise an error if calling a private function from another blueprint:
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    EnforceBlueprintFunctionVisibility,

    // ActorComponents now store their serialization index
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    StoringUCSSerializationIndex,

    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION_PLUS_ONE
    VersionPlusOne,
    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION
    LatestVersion = (FFrameworkObjectVersion::VersionPlusOne as i32) + 1,
}

impl_custom_version_trait!(
    FFrameworkObjectVersion,
    "FFrameworkObjectVersion",
    new_guid(0xCFFC743F, 0x43B04480, 0x939114DF, 0x171D2073),
    VER_UE4_AUTOMATIC_VERSION: LatestVersion,
    VER_UE4_AUTOMATIC_VERSION_PLUS_ONE: VersionPlusOne,
    VER_UE4_ADDED_PACKAGE_OWNER: StoringUCSSerializationIndex,
    VER_UE4_FIX_WIDE_STRING_CRC: BlueprintGeneratedClassIsAlwaysAuthoritative,
    VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID: EditableEventsUseConstRefParameters,
    VER_UE4_POINTLIGHT_SOURCE_ORIENTATION: UserDefinedStructsStoreDefaultInstance,
    VER_UE4_ADDED_SOFT_OBJECT_PATH: PinsStoreFName,
    VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG: LocalVariablesBlueprintVisible,
    VER_UE4_64BIT_EXPORTMAP_SERIALSIZES: HardSoundReferences,
    VER_UE4_ADDED_SEARCHABLE_NAMES: ChangeAudioComponentOverrideSubtitlePriorityDefault,
    VER_UE4_TEMPLATE_INDEX_IN_COOKED_EXPORTS: GeometryCacheMissingMaterials,
    VER_UE4_COMPRESSED_SHADER_RESOURCES: MoveCurveTypesToSkeleton,
    VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR: WheelOffsetIsFromWheel,
    VER_UE4_NAME_HASHES_SERIALIZED: AddSourceReferenceSkeletonToRig,
    VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG: AnimBlueprintSubgraphFix,
    VER_UE4_STREAMABLE_TEXTURE_AABB: UseBodySetupCollisionProfile,
    VER_UE4_OLDEST_LOADABLE_PACKAGE: BeforeCustomVersionWasAdded
);

//
// Custom serialization version for changes made in Dev-Core stream.
#[derive(IntoPrimitive)]
#[repr(i32)]
pub enum FCoreObjectVersion {
    // Before any version changes were made
    // Introduced: UE4Version.VER_UE4_OLDEST_LOADABLE_PACKAGE
    BeforeCustomVersionWasAdded = 0,

    // Introduced: UE4Version.VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG
    MaterialInputNativeSerialize,

    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    EnumProperties,

    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    SkeletalMaterialEditorDataStripping,

    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    FProperties,

    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION_PLUS_ONE
    VersionPlusOne,
    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION
    LatestVersion = (FCoreObjectVersion::VersionPlusOne as i32) + 1,
}

impl_custom_version_trait!(
    FCoreObjectVersion,
    "FCoreObjectVersion",
    new_guid(0x375EC13C, 0x06E448FB, 0xB50084F0, 0x262A717E),
    VER_UE4_AUTOMATIC_VERSION: LatestVersion,
    VER_UE4_AUTOMATIC_VERSION_PLUS_ONE: VersionPlusOne,
    VER_UE4_ADDED_PACKAGE_OWNER: FProperties,
    VER_UE4_FIX_WIDE_STRING_CRC: SkeletalMaterialEditorDataStripping,
    VER_UE4_ADDED_SEARCHABLE_NAMES: EnumProperties,
    VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG: MaterialInputNativeSerialize,
    VER_UE4_OLDEST_LOADABLE_PACKAGE: BeforeCustomVersionWasAdded
);

//
// Custom serialization version for changes made in Dev-Editor stream.
#[derive(IntoPrimitive)]
#[repr(i32)]
pub enum FEditorObjectVersion {
    // Before any version changes were made
    // Introduced: UE4Version.VER_UE4_OLDEST_LOADABLE_PACKAGE
    BeforeCustomVersionWasAdded = 0,

    // Localizable text gathered and stored in packages is now flagged with a localizable text gathering process version
    // Introduced: UE4Version.VER_UE4_STREAMABLE_TEXTURE_AABB
    GatheredTextProcessVersionFlagging,

    // Fixed several issues with the gathered text cache stored in package headers
    // Introduced: UE4Version.VER_UE4_NAME_HASHES_SERIALIZED
    GatheredTextPackageCacheFixesV1,

    // Added support for "root" meta-data (meta-data not associated with a particular object in a package)
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    RootMetaDataSupport,

    // Fixed issues with how Blueprint bytecode was cached
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    GatheredTextPackageCacheFixesV2,

    // Updated FFormatArgumentData to allow variant data to be marshaled from a BP into C++
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    TextFormatArgumentDataIsVariant,

    // Changes to SplineComponent
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    SplineComponentCurvesInStruct,

    // Updated ComboBox to support toggling the menu open, better controller support
    // Introduced: UE4Version.VER_UE4_COMPRESSED_SHADER_RESOURCES
    ComboBoxControllerSupportUpdate,

    // Refactor mesh editor materials
    // Introduced: UE4Version.VER_UE4_COMPRESSED_SHADER_RESOURCES
    RefactorMeshEditorMaterials,

    // Added UFontFace assets
    // Introduced: UE4Version.VER_UE4_TemplateIndex_IN_COOKED_EXPORTS
    AddedFontFaceAssets,

    // Add UPROPERTY for TMap of Mesh section, so the serialize will be done normally (and export to text will work correctly)
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    UPropertryForMeshSection,

    // Update the schema of all widget blueprints to use the WidgetGraphSchema
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    WidgetGraphSchema,

    // Added a specialized content slot to the background blur widget
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    AddedBackgroundBlurContentSlot,

    // Updated UserDefinedEnums to have stable keyed display names
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    StableUserDefinedEnumDisplayNames,

    // Added "Inline" option to UFontFace assets
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    AddedInlineFontFaceAssets,

    // Fix a serialization issue with static mesh FMeshSectionInfoMap FProperty
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    UPropertryForMeshSectionSerialize,

    // Adding a version bump for the new fast widget construction in case of problems.
    // Introduced: UE4Version.VER_UE4_64BIT_EXPORTMAP_SERIALSIZES
    FastWidgetTemplates,

    // Update material thumbnails to be more intelligent on default primitive shape for certain material types
    // Introduced: UE4Version.VER_UE4_64BIT_EXPORTMAP_SERIALSIZES
    MaterialThumbnailRenderingChanges,

    // Introducing a new clipping system for Slate/UMG
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    NewSlateClippingSystem,

    // MovieScene Meta Data added as native Serialization
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    MovieSceneMetaDataSerialization,

    // Text gathered from properties now adds two variants: a version without the package localization ID (for use at runtime), and a version with it (which is editor-only)
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    GatheredTextEditorOnlyPackageLocId,

    // Added AlwaysSign to FNumberFormattingOptions
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    AddedAlwaysSignNumberFormattingOption,

    // Added additional objects that must be serialized as part of this new material feature
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    AddedMaterialSharedInputs,

    // Added morph target section indices
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    AddedMorphTargetSectionIndices,

    // Serialize the instanced static mesh render data, to avoid building it at runtime
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    SerializeInstancedStaticMeshRenderData,

    // Change to MeshDescription serialization (moved to release)
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    MeshDescriptionNewSerializationMovedToRelease,

    // New format for mesh description attributes
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    MeshDescriptionNewAttributeFormat,

    // Switch root component of SceneCapture actors from MeshComponent to SceneComponent
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    ChangeSceneCaptureRootComponent,

    // StaticMesh serializes MeshDescription instead of RawMesh
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    StaticMeshDeprecatedRawMesh,

    // MeshDescriptionBulkData contains a Guid used as a DDC key
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    MeshDescriptionBulkDataGuid,

    // Change to MeshDescription serialization (removed FMeshPolygon::HoleContours)
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    MeshDescriptionRemovedHoles,

    // Change to the WidgetCompoent WindowVisibilty default value
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    ChangedWidgetComponentWindowVisibilityDefault,

    // Avoid keying culture invariant display strings during serialization to avoid non-deterministic cooking issues
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    CultureInvariantTextSerializationKeyStability,

    // Change to UScrollBar and UScrollBox thickness property (removed implicit padding of 2, so thickness value must be incremented by 4).
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    ScrollBarThicknessChange,

    // Deprecated LandscapeHoleMaterial
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    RemoveLandscapeHoleMaterial,

    // MeshDescription defined by triangles instead of arbitrary polygons
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    MeshDescriptionTriangles,

    // Add weighted area and angle when computing the normals
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    ComputeWeightedNormals,

    // SkeletalMesh now can be rebuild in editor, no more need to re-import
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    SkeletalMeshBuildRefactor,

    // Move all SkeletalMesh source data into a private uasset in the same package has the skeletalmesh
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    SkeletalMeshMoveEditorSourceDataToPrivateAsset,

    // Parse text only if the number is inside the limits of its type
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    NumberParsingOptionsNumberLimitsAndClamping,

    // Make sure we can have more then 255 material in the skeletal mesh source data
    // Introduced: UE4Version.VER_UE4_NON_OUTER_PACKAGE_IMPORT
    SkeletalMeshSourceDataSupport16bitOfMaterialNumber,

    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION_PLUS_ONE
    VersionPlusOne,
    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION
    LatestVersion = (FEditorObjectVersion::VersionPlusOne as i32) + 1,
}

impl_custom_version_trait!(
    FEditorObjectVersion,
    "FEditorObjectVersion",
    new_guid(0xE4B068ED, 0xF49442E9, 0xA231DA0B, 0x2E46BB41),
    VER_UE4_AUTOMATIC_VERSION: LatestVersion,
    VER_UE4_AUTOMATIC_VERSION_PLUS_ONE: VersionPlusOne,
    VER_UE4_NON_OUTER_PACKAGE_IMPORT: SkeletalMeshSourceDataSupport16bitOfMaterialNumber,
    VER_UE4_ADDED_PACKAGE_OWNER: SkeletalMeshMoveEditorSourceDataToPrivateAsset,
    VER_UE4_FIX_WIDE_STRING_CRC: MeshDescriptionTriangles,
    VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID: MeshDescriptionNewAttributeFormat,
    VER_UE4_ADDED_SOFT_OBJECT_PATH: AddedAlwaysSignNumberFormattingOption,
    VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG: GatheredTextEditorOnlyPackageLocId,
    VER_UE4_64BIT_EXPORTMAP_SERIALSIZES: MaterialThumbnailRenderingChanges,
    VER_UE4_ADDED_SEARCHABLE_NAMES: UPropertryForMeshSectionSerialize,
    VER_UE4_TEMPLATE_INDEX_IN_COOKED_EXPORTS: AddedFontFaceAssets,
    VER_UE4_COMPRESSED_SHADER_RESOURCES: RefactorMeshEditorMaterials,
    VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR: SplineComponentCurvesInStruct,
    VER_UE4_NAME_HASHES_SERIALIZED: GatheredTextPackageCacheFixesV1,
    VER_UE4_STREAMABLE_TEXTURE_AABB: GatheredTextProcessVersionFlagging,
    VER_UE4_OLDEST_LOADABLE_PACKAGE: BeforeCustomVersionWasAdded
);

//
// Custom serialization version for changes made in Dev-AnimPhys stream
#[derive(IntoPrimitive)]
#[repr(i32)]
pub enum FAnimPhysObjectVersion {
    // Before any version changes were made
    // Introduced: UE4Version.VER_UE4_OLDEST_LOADABLE_PACKAGE
    BeforeCustomVersionWasAdded,

    // convert animnode look at to use just default axis instead of enum, which doesn't do much
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    ConvertAnimNodeLookAtAxis,

    // Change FKSphylElem and FKBoxElem to use Rotators not Quats for easier editing
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    BoxSphylElemsUseRotators,

    // Change thumbnail scene info and asset import data to be transactional
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    ThumbnailSceneInfoAndAssetImportDataAreTransactional,

    // Enabled clothing masks rather than painting parameters directly
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    AddedClothingMaskWorkflow,

    // Remove UID from smart name serialize, it just breaks determinism
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    RemoveUIDFromSmartNameSerialize,

    // Convert FName Socket to FSocketReference and added TargetReference that support bone and socket
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    CreateTargetReference,

    // Tune soft limit stiffness and damping coefficients
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    TuneSoftLimitStiffnessAndDamping,

    // Fix possible inf/nans in clothing particle masses
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    FixInvalidClothParticleMasses,

    // Moved influence count to cached data
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    CacheClothMeshInfluences,

    // Remove GUID from Smart Names entirely + remove automatic name fixup
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    SmartNameRefactorForDeterministicCooking,

    // rename the variable and allow individual curves to be set
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    RenameDisableAnimCurvesToAllowAnimCurveEvaluation,

    // link curve to LOD, so curve metadata has to include LODIndex
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    AddLODToCurveMetaData,

    // Fixed blend profile references persisting after paste when they aren't compatible
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    FixupBadBlendProfileReferences,

    // Allowing multiple audio plugin settings
    // Introduced: UE4Version.VER_UE4_ADDED_SOFT_OBJECT_PATH
    AllowMultipleAudioPluginSettings,

    // Change RetargetSource reference to SoftObjectPtr
    // Introduced: UE4Version.VER_UE4_POINTLIGHT_SOURCE_ORIENTATION
    ChangeRetargetSourceReferenceToSoftObjectPtr,

    // Save editor only full pose for pose asset
    // Introduced: UE4Version.VER_UE4_POINTLIGHT_SOURCE_ORIENTATION
    SaveEditorOnlyFullPoseForPoseAsset,

    // Asset change and cleanup to facilitate new streaming system
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    GeometryCacheAssetDeprecation,

    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION_PLUS_ONE
    VersionPlusOne,
    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION
    LatestVersion = (FAnimPhysObjectVersion::VersionPlusOne as i32) + 1,
}

impl_custom_version_trait!(
    FAnimPhysObjectVersion,
    "FAnimPhysObjectVersion",
    new_guid(0x29E575DD, 0xE0A34627, 0x9D10D276, 0x232CDCEA),
    VER_UE4_AUTOMATIC_VERSION: LatestVersion,
    VER_UE4_AUTOMATIC_VERSION_PLUS_ONE: VersionPlusOne,
    VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID: GeometryCacheAssetDeprecation,
    VER_UE4_POINTLIGHT_SOURCE_ORIENTATION: SaveEditorOnlyFullPoseForPoseAsset,
    VER_UE4_ADDED_SOFT_OBJECT_PATH: AllowMultipleAudioPluginSettings,
    VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG: CacheClothMeshInfluences,
    VER_UE4_OLDEST_LOADABLE_PACKAGE: BeforeCustomVersionWasAdded
);

//
// Custom serialization version for changes made in Release streams.
#[derive(IntoPrimitive)]
#[repr(i32)]
pub enum FReleaseObjectVersion {
    // Before any version changes were made
    // Introduced: UE4Version.VER_UE4_OLDEST_LOADABLE_PACKAGE
    BeforeCustomVersionWasAdded = 0,

    // Static Mesh extended bounds radius fix
    // Introduced: UE4Version.VER_UE4_NAME_HASHES_SERIALIZED
    StaticMeshExtendedBoundsFix,

    // Physics asset bodies are either in the sync scene or the async scene, but not both
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    NoSyncAsyncPhysAsset,

    // ULevel was using TTransArray incorrectly (serializing the entire array in addition to individual mutations). converted to a TArray
    // Introduced: UE4Version.VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
    LevelTransArrayConvertedToTArray,

    // Add Component node templates now use their own unique naming scheme to ensure more reliable archetype lookups.
    // Introduced: UE4Version.VER_UE4_TemplateIndex_IN_COOKED_EXPORTS
    AddComponentNodeTemplateUniqueNames,

    // Fix a serialization issue with static mesh FMeshSectionInfoMap FProperty
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    UPropertryForMeshSectionSerialize,

    // Existing HLOD settings screen size to screen area conversion
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    ConvertHLODScreenSize,

    // Adding mesh section info data for existing billboard LOD models
    // Introduced: UE4Version.VER_UE4_ADDED_SEARCHABLE_NAMES
    SpeedTreeBillboardSectionInfoFixup,

    // Change FMovieSceneEventParameters::StructType to be a string asset reference from a TWeakObjectPtr UScriptStruct
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    EventSectionParameterStringAssetRef,

    // Remove serialized irradiance map data from skylight.
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    SkyLightRemoveMobileIrradianceMap,

    // rename bNoTwist to bAllowTwist
    // Introduced: UE4Version.VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
    RenameNoTwistToAllowTwistInTwoBoneIK,

    // Material layers serialization refactor
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    MaterialLayersParameterSerializationRefactor,

    // Added disable flag to skeletal mesh data
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    AddSkeletalMeshSectionDisable,

    // Removed objects that were serialized as part of this material feature
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    RemovedMaterialSharedInputCollection,

    // HISMC Cluster Tree migration to add new data
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    HISMCClusterTreeMigration,

    // Default values on pins in blueprints could be saved incoherently
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    PinDefaultValuesVerified,

    // During copy and paste transition getters could end up with broken state machine references
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    FixBrokenStateMachineReferencesInTransitionGetters,

    // Change to MeshDescription serialization
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    MeshDescriptionNewSerialization,

    // Change to not clamp RGB values > 1 on linear color curves
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    UnclampRGBColorCurves,

    // Bugfix for FAnimObjectVersion::LinkTimeAnimBlueprintRootDiscovery.
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    LinkTimeAnimBlueprintRootDiscoveryBugFix,

    // Change trail anim node variable deprecation
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
    TrailNodeBlendVariableNameChange,

    // Make sure the Blueprint Replicated Property Conditions are actually serialized properly.
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    PropertiesSerializeRepCondition,

    // DepthOfFieldFocalDistance at 0 now disables DOF instead of DepthOfFieldFstop at 0.
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    FocalDistanceDisablesDOF,

    // Removed versioning, but version entry must still exist to keep assets saved with this version loadable
    // Introduced: UE4Version.VER_UE4_FIX_WIDE_STRING_CRC
    UnusedSoundClass2dreverbSend,

    // Groom asset version
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    GroomAssetVersion1,

    // Groom asset version
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    GroomAssetVersion2,

    // Store applied version of Animation Modifier to use when reverting
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    SerializeAnimModifierState,

    // Groom asset version
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    GroomAssetVersion3,

    // Upgrade filmback
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    DeprecateFilmbackSettings,

    // custom collision type
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    CustomImplicitCollisionType,

    // FFieldPath will serialize the owner struct reference and only a short path to its property
    // Introduced: UE4Version.VER_UE4_ADDED_PACKAGE_OWNER
    FFieldPathOwnerSerialization,

    // Dummy version to allow us to fix up the fact that ReleaseObjectVersion was changed elsewhere
    // Introduced: UE4Version.VER_UE4_SKINWEIGHT_PROFILE_DATA_LAYOUT_CHANGES
    ReleaseObjectVersionFixup,

    // Pin types include a flag that propagates the 'CPF_UObjectWrapper' flag to generated properties
    // Introduced: UE4Version.VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS
    PinTypeIncludesUObjectWrapperFlag,

    // Added Weight member to FMeshToMeshVertData
    // Introduced: UE4Version.VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS
    WeightFMeshToMeshVertData,

    // Animation graph node bindings displayed as pins
    // Introduced: UE4Version.VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS
    AnimationGraphNodeBindingsDisplayedAsPins,

    // Serialized rigvm offset segment paths
    // Introduced: UE4Version.VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS
    SerializeRigVMOffsetSegmentPaths,

    // Upgrade AbcGeomCacheImportSettings for velocities
    // Introduced: UE4Version.VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS
    AbcVelocitiesSupport,

    // Add margin support to Chaos Convex
    // Introduced: UE4Version.VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS
    MarginAddedToConvexAndBox,

    // Add structure data to Chaos Convex
    // Introduced: UE4Version.VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS
    StructureDataAddedToConvex,

    // Changed axis UI for LiveLink AxisSwitch Pre Processor
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    AddedFrontRightUpAxesToLiveLinkPreProcessor,

    // Some sequencer event sections that were copy-pasted left broken links to the director BP
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    FixupCopiedEventSections,

    // Serialize the number of bytes written when serializing function arguments
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    RemoteControlSerializeFunctionArgumentsSize,

    // Add loop counters to sequencer's compiled sub-sequence data
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    AddedSubSequenceEntryWarpCounter,

    // Remove default resolution limit of 512 pixels for cubemaps generated from long-lat sources
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    LonglatTextureCubeDefaultMaxResolution,

    // bake center of mass into chaos cache
    // Introduced: UE4Version.VER_UE4_CORRECT_LICENSEE_FLAG
    GeometryCollectionCacheRemovesMassToLocal,

    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION_PLUS_ONE
    VersionPlusOne,
    // Introduced: UE4Version.VER_UE4_AUTOMATIC_VERSION
    LatestVersion = (FReleaseObjectVersion::VersionPlusOne as i32) + 1,
}

impl_custom_version_trait!(
    FReleaseObjectVersion,
    "FReleaseObjectVersion",
    new_guid(0x9C54D522, 0xA8264FBE, 0x94210746, 0x61B482D0),
    VER_UE4_AUTOMATIC_VERSION: LatestVersion,
    VER_UE4_AUTOMATIC_VERSION_PLUS_ONE: VersionPlusOne,
    VER_UE4_CORRECT_LICENSEE_FLAG: GeometryCollectionCacheRemovesMassToLocal,
    VER_UE4_ASSETREGISTRY_DEPENDENCYFLAGS: StructureDataAddedToConvex,
    VER_UE4_SKINWEIGHT_PROFILE_DATA_LAYOUT_CHANGES: ReleaseObjectVersionFixup,
    VER_UE4_ADDED_PACKAGE_OWNER: FFieldPathOwnerSerialization,
    VER_UE4_FIX_WIDE_STRING_CRC: UnusedSoundClass2dreverbSend,
    VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID: TrailNodeBlendVariableNameChange,
    VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG: RenameNoTwistToAllowTwistInTwoBoneIK,
    VER_UE4_ADDED_SEARCHABLE_NAMES: SpeedTreeBillboardSectionInfoFixup,
    VER_UE4_TEMPLATE_INDEX_IN_COOKED_EXPORTS: AddComponentNodeTemplateUniqueNames,
    VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR: LevelTransArrayConvertedToTArray,
    VER_UE4_NAME_HASHES_SERIALIZED: StaticMeshExtendedBoundsFix,
    VER_UE4_OLDEST_LOADABLE_PACKAGE: BeforeCustomVersionWasAdded
);

#[derive(IntoPrimitive, TryFromPrimitive, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
#[repr(i32)]
pub enum FAssetRegistryVersionType {
    PreVersioning = 0,     // From before file versioning was implemented
    HardSoftDependencies, // The first version of the runtime asset registry to include file versioning.
    AddAssetRegistryState, // Added FAssetRegistryState and support for piecemeal serialization
    ChangedAssetData, // AssetData serialization format changed, versions before this are not readable
    RemovedMD5Hash,   // Removed MD5 hash from package data
    AddedHardManage,  // Added hard/soft manage references
    AddedCookedMD5Hash, // Added MD5 hash of cooked package to package data
    AddedDependencyFlags, // Added UE::AssetRegistry::EDependencyProperty to each dependency
    FixedTags,        // Major tag format change that replaces USE_COMPACT_ASSET_REGISTRY:
    // * Target tag INI settings cooked into tag data
    // * Instead of FString values are stored directly as one of:
    //		- Narrow / wide string
    //		- [Numberless] FName
    //		- [Numberless] export path
    //		- Localized string
    // * All value types are deduplicated
    // * All key-value maps are cooked into a single contiguous range
    // * Switched from FName table to seek-free and more optimized FName batch loading
    // * Removed global tag storage, a tag map reference-counts one store per asset registry
    // * All configs can mix fixed and loose tag maps
    WorkspaceDomain,                 // Added Version information to AssetPackageData
    PackageImportedClasses,          // Added ImportedClasses to AssetPackageData
    PackageFileSummaryVersionChange, // A new version number of UE5 was added to FPackageFileSummary
    ObjectResourceOptionalVersionChange, // Change to linker export/import resource serialization
    AddedChunkHashes, // Added FIoHash for each FIoChunkId in the package to the AssetPackageData.
    ClassPaths, // Classes are serialized as path names rather than short object names, e.g. /Script/Engine.StaticMesh

    // -----<new versions can be added above this line>-------------------------------------------------
    LatestVersion,
    VersionPlusOne,
}

lazy_static! {
    static ref ASSET_REGISTRY_VERSION_GUID: Guid =
        new_guid(0x717F9EE7, 0xE9B0493A, 0x88B39132, 0x1B388107);
}

impl FAssetRegistryVersionType {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let mut guid = [0u8; 16];
        asset.read_exact(&mut guid)?;

        if guid == *ASSET_REGISTRY_VERSION_GUID {
            return Ok(Self::try_from(asset.read_i32::<LittleEndian>()?)?);
        }

        Ok(FAssetRegistryVersionType::LatestVersion)
    }

    pub fn write<Writer: AssetWriter>(&self, writer: &mut Writer) -> Result<(), Error> {
        writer.write_all(&(*ASSET_REGISTRY_VERSION_GUID))?;
        writer.write_i32::<LittleEndian>((*self).into())?;
        Ok(())
    }
}

impl Display for FAssetRegistryVersionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            FAssetRegistryVersionType::PreVersioning => write!(f, "PreVersioning"),
            FAssetRegistryVersionType::HardSoftDependencies => write!(f, "HardSoftDependencies"),
            FAssetRegistryVersionType::AddAssetRegistryState => write!(f, "AddAssetRegistryState"),
            FAssetRegistryVersionType::ChangedAssetData => write!(f, "ChangedAssetData"),
            FAssetRegistryVersionType::RemovedMD5Hash => write!(f, "RemovedMD5Hash"),
            FAssetRegistryVersionType::AddedHardManage => write!(f, "AddedHardManage"),
            FAssetRegistryVersionType::AddedCookedMD5Hash => write!(f, "AddedCookedMD5Hash"),
            FAssetRegistryVersionType::AddedDependencyFlags => write!(f, "AddedDependencyFlags"),
            FAssetRegistryVersionType::FixedTags => write!(f, "FixedTags"),
            FAssetRegistryVersionType::WorkspaceDomain => write!(f, "WorkspaceDomain"),
            FAssetRegistryVersionType::PackageImportedClasses => {
                write!(f, "PackageImportedClasses")
            }
            FAssetRegistryVersionType::PackageFileSummaryVersionChange => {
                write!(f, "PackageFileSummaryVersionChange")
            }
            FAssetRegistryVersionType::ObjectResourceOptionalVersionChange => {
                write!(f, "ObjectResourceOptionalVersionChange")
            }
            FAssetRegistryVersionType::AddedChunkHashes => write!(f, "AddedChunkHashes"),
            FAssetRegistryVersionType::ClassPaths => write!(f, "ClassPaths"),
            FAssetRegistryVersionType::LatestVersion => write!(f, "LatestVersion"),
            FAssetRegistryVersionType::VersionPlusOne => write!(f, "VersionPlusOne"),
        }
    }
}
