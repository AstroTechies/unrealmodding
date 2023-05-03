//! Generic unreal asset traits
//! Must be implemented for all unreal assets

use unreal_asset_proc_macro::FNameContainer;

use crate::{
    containers::{indexed_map::IndexedMap, shared_resource::SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::{get_object_versions, guess_engine_version, EngineVersion},
    exports::Export,
    flags::EPackageFlags,
    object_version::{ObjectVersion, ObjectVersionUE5},
    properties::world_tile_property::FWorldTileInfo,
    types::{fname::FName, PackageIndex},
    unversioned::Usmap,
};

use self::name_map::NameMap;

pub mod cityhash64_string_map;
pub mod name_map;
/// Unreal asset data, this is relevant for all assets
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq)]
pub struct AssetData {
    /// Does asset use the event driven loader
    pub use_event_driven_loader: bool,
    /// Is asset unversioned
    pub unversioned: bool,
    /// Asset flags
    #[container_ignore]
    pub package_flags: EPackageFlags,

    /// File licensee version, used by some games for their own engine versioning.
    pub file_license_version: i32,

    /// Object version
    #[container_ignore]
    pub object_version: ObjectVersion,
    /// UE5 object version
    #[container_ignore]
    pub object_version_ue5: ObjectVersionUE5,

    /// Custom versions
    #[container_ignore]
    pub custom_versions: Vec<CustomVersion>,

    /// .usmap mappings
    #[container_ignore]
    pub mappings: Option<Usmap>,

    /// Object exports
    pub exports: Vec<Export>,

    /// World tile information used by WorldComposition
    /// Degines propertiesn ecessary for tile positioning in the world
    pub world_tile_info: Option<FWorldTileInfo>,

    /// Map properties with StructProperties inside, have no way of determining the underlying type of the struct
    /// This is used for specifying those types for keys
    #[container_ignore]
    pub map_key_override: IndexedMap<String, String>,
    /// Map properties with StructProperties inside, have no way of determining the underlying type of the struct
    /// This is used for specifying those types for values
    #[container_ignore]
    pub map_value_override: IndexedMap<String, String>,

    /// Array properties with StructProperties inside, have no way of determining the underlying type of the struct
    /// This is used for specifying those types
    #[container_ignore]
    pub array_struct_type_override: IndexedMap<String, String>,
}

impl AssetData {
    /// Creates a new `AssetData` instance
    pub fn new() -> AssetData {
        AssetData::default()
    }

    /// Set asset engine version
    pub fn set_engine_version(&mut self, engine_version: EngineVersion) {
        if engine_version == EngineVersion::UNKNOWN {
            return;
        }

        let (object_version, object_version_ue5) = get_object_versions(engine_version);

        self.object_version = object_version;
        self.object_version_ue5 = object_version_ue5;
        self.custom_versions = CustomVersion::get_default_custom_version_container(engine_version);
    }

    /// Get a custom version from this AssetData
    ///
    /// # Example
    ///
    /// ```no_run,ignore
    /// use unreal_asset::{
    ///     asset::AssetData,
    ///     custom_version::FFrameworkObjectVersion,
    /// };
    /// let data: AssetData = ...;
    /// println!("{:?}", data.get_custom_version::<FFrameworkObjectVersion>());
    /// ```
    pub fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>,
    {
        self.custom_versions
            .iter()
            .find(|e| {
                e.friendly_name
                    .as_ref()
                    .map(|name| name == T::FRIENDLY_NAME)
                    .unwrap_or(false)
            })
            .cloned()
            .unwrap_or_else(|| CustomVersion::new(T::GUID, 0))
    }

    /// Get engine version
    pub fn get_engine_version(&self) -> EngineVersion {
        guess_engine_version(
            self.object_version,
            self.object_version_ue5,
            &self.custom_versions,
        )
    }

    /// Get an export
    pub fn get_export(&self, index: PackageIndex) -> Option<&Export> {
        if !index.is_export() {
            return None;
        }

        let index = index.index - 1;

        if index < 0 || index >= self.exports.len() as i32 {
            return None;
        }

        Some(&self.exports[index as usize])
    }

    /// Get a mutable export reference
    pub fn get_export_mut(&mut self, index: PackageIndex) -> Option<&mut Export> {
        if !index.is_export() {
            return None;
        }

        let index = index.index - 1;

        if index < 0 || index >= self.exports.len() as i32 {
            return None;
        }

        Some(&mut self.exports[index as usize])
    }
}

impl Default for AssetData {
    fn default() -> Self {
        Self {
            use_event_driven_loader: false,
            unversioned: true,
            package_flags: EPackageFlags::PKG_NONE,
            file_license_version: 0,
            object_version: ObjectVersion::UNKNOWN,
            object_version_ue5: ObjectVersionUE5::UNKNOWN,
            custom_versions: Vec::new(),
            mappings: None,
            exports: Vec::new(),
            world_tile_info: None,
            map_key_override: IndexedMap::from([
                ("PlayerCharacterIDs".to_string(), "Guid".to_string()),
                (
                    "m_PerConditionValueToNodeMap".to_string(),
                    "Guid".to_string(),
                ),
                ("BindingIdToReferences".to_string(), "Guid".to_string()),
                (
                    "UserParameterRedirects".to_string(),
                    "NiagaraVariable".to_string(),
                ),
                (
                    "Tracks".to_string(),
                    "MovieSceneTrackIdentifier".to_string(),
                ),
                (
                    "SubSequences".to_string(),
                    "MovieSceneSequenceID".to_string(),
                ),
                ("Hierarchy".to_string(), "MovieSceneSequenceID".to_string()),
                (
                    "TrackSignatureToTrackIdentifier".to_string(),
                    "Guid".to_string(),
                ),
                ("ItemsToRefund".to_string(), "Guid".to_string()),
                ("PlayerCharacterIDMap".to_string(), "Guid".to_string()),
            ]),
            map_value_override: IndexedMap::from([
                ("ColorDatabase".to_string(), "LinearColor".to_string()),
                (
                    "UserParameterRedirects".to_string(),
                    "NiagaraVariable".to_string(),
                ),
                (
                    "TrackSignatureToTrackIdentifier".to_string(),
                    "MovieSceneTrackIdentifier".to_string(),
                ),
                (
                    "RainChanceMinMaxPerWeatherState".to_string(),
                    "FloatRange".to_string(),
                ),
            ]),
            array_struct_type_override: IndexedMap::from([(
                "Keys".to_string(),
                "RichCurveKey".to_string(),
            )]),
        }
    }
}

/// Unreal asset trait, must be implemented for all assets
pub trait AssetTrait {
    /// Gets a reference to the asset data
    fn get_asset_data(&self) -> &AssetData;
    /// Gets a mutable reference to the asset data
    fn get_asset_data_mut(&mut self) -> &mut AssetData;

    /// Gets the name map
    fn get_name_map(&self) -> SharedResource<NameMap>;

    // todo: hese methods probably should be replaced with getters to name map
    /// Search an FName reference
    fn search_name_reference(&self, name: &str) -> Option<i32>;

    /// Add an FName reference
    fn add_name_reference(&mut self, name: String, force_add_duplicates: bool) -> i32;

    /// Get a name reference by an FName map index
    fn get_name_reference(&self, index: i32) -> String;

    /// Add an `FName`
    fn add_fname(&mut self, slice: &str) -> FName;
}
