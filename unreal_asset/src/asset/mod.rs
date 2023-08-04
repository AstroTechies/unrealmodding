//! Generic unreal asset traits
//! Must be implemented for all unreal assets

use std::io::SeekFrom;

use unreal_asset_proc_macro::FNameContainer;

use crate::{
    cast,
    containers::{indexed_map::IndexedMap, shared_resource::SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::{get_object_versions, EngineVersion},
    error::Error,
    exports::{
        base_export::BaseExport, class_export::ClassExport, data_table_export::DataTableExport,
        enum_export::EnumExport, function_export::FunctionExport, level_export::LevelExport,
        normal_export::NormalExport, property_export::PropertyExport, raw_export::RawExport,
        string_table_export::StringTableExport,
        user_defined_struct_export::UserDefinedStructExport, world_export::WorldExport, Export,
        ExportNormalTrait,
    },
    flags::EPackageFlags,
    fproperty::FProperty,
    object_version::{ObjectVersion, ObjectVersionUE5},
    properties::world_tile_property::FWorldTileInfo,
    reader::archive_reader::ArchiveReader,
    types::{fname::FName, PackageIndex},
    unversioned::Usmap,
};

use self::name_map::NameMap;

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
    pub engine_version: EngineVersion,
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

/// Export read from [`AssetData`]
///
/// To get the actual export, call `.reduce()`
///
/// This is needed because export reading may want to modify [`AssetData`] which upsets the borrow checker
#[derive(Debug, Clone)]
pub struct ReadExport {
    export: Export,
    new_map_key_overrides: IndexedMap<String, String>,
    new_map_value_overrides: IndexedMap<String, String>,
    new_array_overrides: IndexedMap<String, String>,
}

impl ReadExport {
    /// Create a new `ReadExport` instance
    pub fn new(
        export: Export,
        new_map_key_overrides: IndexedMap<String, String>,
        new_map_value_overrides: IndexedMap<String, String>,
        new_array_overrides: IndexedMap<String, String>,
    ) -> Self {
        ReadExport {
            export,
            new_map_key_overrides,
            new_map_value_overrides,
            new_array_overrides,
        }
    }

    /// Reduce `ReadExport` to an [`Export`]
    pub fn reduce(self, asset_data: &mut AssetData) -> Export {
        asset_data.map_key_override.extend(
            self.new_map_key_overrides
                .into_iter()
                .map(|(_, k, v)| (k, v)),
        );
        asset_data.map_value_override.extend(
            self.new_map_value_overrides
                .into_iter()
                .map(|(_, k, v)| (k, v)),
        );
        asset_data
            .array_struct_type_override
            .extend(self.new_array_overrides.into_iter().map(|(_, k, v)| (k, v)));
        self.export
    }
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

        self.engine_version = engine_version;
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
        self.engine_version
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

    /// Searches for an returns this asset's ClassExport, if one exists
    pub fn get_class_export(&self) -> Option<&ClassExport> {
        self.exports
            .iter()
            .find_map(|e| cast!(Export, ClassExport, e))
    }

    /// Get if the asset has unversioned properties
    pub fn has_unversioned_properties(&self) -> bool {
        self.package_flags
            .contains(EPackageFlags::PKG_UNVERSIONED_PROPERTIES)
    }
}

impl Default for AssetData {
    fn default() -> Self {
        Self {
            use_event_driven_loader: false,
            unversioned: true,
            package_flags: EPackageFlags::PKG_NONE,
            file_license_version: 0,
            engine_version: EngineVersion::UNKNOWN,
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

    // todo: these methods probably should be replaced with getters to name map
    /// Search an FName reference
    fn search_name_reference(&self, name: &str) -> Option<i32>;

    /// Add an FName reference
    fn add_name_reference(&mut self, name: String, force_add_duplicates: bool) -> i32;

    /// Get a name reference by an FName map index and do something with it
    fn get_name_reference<T>(&self, index: i32, func: impl FnOnce(&str) -> T) -> T;

    /// Add an `FName`
    fn add_fname(&mut self, slice: &str) -> FName;
}

/// Export reader trait, used to read exports from an asset, implemented for all assets that implemented [`ArchiveReader`]+[`AssetTrait`]
pub trait ExportReaderTrait: ArchiveReader + AssetTrait + Sized {
    /// Read an export from this asset
    ///
    /// This function doesn't automatically create a raw export if an error occurs
    ///
    /// This function also doens't automatically reduce the export
    ///
    /// # Arguments
    ///
    /// * `base_export` - base export used for reading this export
    /// * `i` - export index
    fn read_export_no_raw(
        &mut self,
        base_export: BaseExport,
        i: usize,
    ) -> Result<ReadExport, Error> {
        let asset_data = self.get_asset_data();
        let next_starting = match i < (asset_data.exports.len() - 1) {
            true => match &asset_data.exports[i + 1] {
                Export::BaseExport(next_export) => next_export.serial_offset as u64,
                _ => self.data_length()? - 4,
            },
            false => self.data_length()? - 4,
        };

        self.seek(SeekFrom::Start(base_export.serial_offset as u64))?;

        //todo: manual skips
        let export_class_type = self
            .get_export_class_type(base_export.class_index)
            .ok_or_else(|| Error::invalid_package_index("Unknown class type".to_string()))?;

        let mut new_map_key_overrides = IndexedMap::new();
        let mut new_map_value_overrides = IndexedMap::new();
        let new_array_overrides = IndexedMap::new();

        let mut export: Export = export_class_type.get_content(|class| {
            Ok::<Export, Error>(match class {
                "Level" => LevelExport::from_base(&base_export, self)?.into(),
                "World" => WorldExport::from_base(&base_export, self)?.into(),
                "UserDefinedStruct" => {
                    UserDefinedStructExport::from_base(&base_export, self)?.into()
                }
                "StringTable" => StringTableExport::from_base(&base_export, self)?.into(),
                "Enum" | "UserDefinedEnum" => EnumExport::from_base(&base_export, self)?.into(),
                "Function" => FunctionExport::from_base(&base_export, self)?.into(),
                _ => {
                    if export_class_type.ends_with("DataTable") {
                        DataTableExport::from_base(&base_export, self)?.into()
                    } else if export_class_type.ends_with("StringTable") {
                        StringTableExport::from_base(&base_export, self)?.into()
                    } else if export_class_type.ends_with("BlueprintGeneratedClass") {
                        let class_export = ClassExport::from_base(&base_export, self)?;

                        for entry in &class_export.struct_export.loaded_properties {
                            if let FProperty::FMapProperty(map) = entry {
                                let key_override = match &*map.key_prop {
                                    FProperty::FStructProperty(struct_property) => {
                                        match struct_property.struct_value.is_import() {
                                            true => self
                                                .get_import(struct_property.struct_value)
                                                .map(|e| e.object_name.get_owned_content()),
                                            false => None,
                                        }
                                    }
                                    _ => None,
                                };
                                if let Some(key) = key_override {
                                    new_map_key_overrides
                                        .insert(map.generic_property.name.get_owned_content(), key);
                                }

                                let value_override = match &*map.value_prop {
                                    FProperty::FStructProperty(struct_property) => {
                                        match struct_property.struct_value.is_import() {
                                            true => self
                                                .get_import(struct_property.struct_value)
                                                .map(|e| e.object_name.get_owned_content()),
                                            false => None,
                                        }
                                    }
                                    _ => None,
                                };

                                if let Some(value) = value_override {
                                    new_map_value_overrides.insert(
                                        map.generic_property.name.get_owned_content(),
                                        value,
                                    );
                                }
                            }
                        }
                        class_export.into()
                    } else if export_class_type.ends_with("Property") {
                        PropertyExport::from_base(&base_export, self)?.into()
                    } else {
                        NormalExport::from_base(&base_export, self)?.into()
                    }
                }
            })
        })?;

        let extras_len = next_starting as i64 - self.position() as i64;
        if extras_len < 0 {
            // todo: warning?

            self.seek(SeekFrom::Start(base_export.serial_offset as u64))?;
            let export: Export = RawExport::from_base(base_export, self)?.into();
            return Ok(ReadExport::new(
                export,
                new_map_key_overrides,
                new_map_value_overrides,
                new_array_overrides,
            ));
        } else if let Some(normal_export) = export.get_normal_export_mut() {
            let mut extras = vec![0u8; extras_len as usize];
            self.read_exact(&mut extras)?;
            normal_export.extras = extras;
        }

        Ok(ReadExport::new(
            export,
            new_map_key_overrides,
            new_map_value_overrides,
            new_array_overrides,
        ))
    }

    /// Read an export from this asset
    ///
    /// If an error occurs during export reading, it reads a RawExport and returns that
    ///
    /// This function also automatically reduces the [`ReadExport`] to an [`Export`]
    ///
    /// # Arguments
    ///
    /// * `i` - export index
    fn read_export(&mut self, i: usize) -> Result<Export, Error> {
        let asset_data = self.get_asset_data();
        let base_export =
            cast!(Export, BaseExport, asset_data.exports[i].clone()).ok_or_else(|| {
                Error::invalid_file("Couldn't cast to BaseExport when reading exports".to_string())
            })?;

        let serial_offset = base_export.serial_offset as u64;

        match self.read_export_no_raw(base_export.clone(), i) {
            Ok(e) => {
                let asset_data_mut = self.get_asset_data_mut();
                let reduced = e.reduce(asset_data_mut);

                Ok(reduced)
            }
            Err(_e) => {
                // todo: warning?
                self.seek(SeekFrom::Start(serial_offset))?;
                Ok(RawExport::from_base(base_export, self)?.into())
            }
        }
    }
}

impl<R: ArchiveReader + AssetTrait + Sized> ExportReaderTrait for R {}
