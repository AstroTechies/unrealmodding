//! Generic unreal asset traits
//! Must be implemented for all unreal assets

use std::io::SeekFrom;

use unreal_asset_base::{
    cast,
    containers::{indexed_map::IndexedMap, name_map::NameMap, shared_resource::SharedResource},
    custom_version::{CustomVersion, CustomVersionTrait},
    engine_version::{get_object_versions, EngineVersion},
    error::Error,
    flags::EPackageFlags,
    object_version::{ObjectVersion, ObjectVersionUE5},
    reader::ArchiveReader,
    types::{FName, PackageIndex, PackageIndexTrait},
    unversioned::Usmap,
    FNameContainer,
};
use unreal_asset_exports::{
    base_export::BaseExport, class_export::ClassExport, data_table_export::DataTableExport,
    enum_export::EnumExport, function_export::FunctionExport, level_export::LevelExport,
    normal_export::NormalExport, properties::fproperty::FProperty, property_export::PropertyExport,
    raw_export::RawExport, string_table_export::StringTableExport,
    user_defined_struct_export::UserDefinedStructExport, world_export::WorldExport, Export,
    ExportNormalTrait,
};
use unreal_asset_properties::world_tile_property::FWorldTileInfo;

use crate::package_file_summary::PackageFileSummary;

/// Unreal asset data, this is relevant for all assets
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq)]
pub struct AssetData<Index: PackageIndexTrait> {
    /// Does asset use the event driven loader
    pub use_event_driven_loader: bool,
    /// Package file summary
    #[container_ignore]
    pub summary: PackageFileSummary,

    /// Object version
    #[container_ignore]
    pub engine_version: EngineVersion,
    /// Object version
    #[container_ignore]
    pub object_version: ObjectVersion,
    /// UE5 object version
    #[container_ignore]
    pub object_version_ue5: ObjectVersionUE5,

    /// .usmap mappings
    #[container_ignore]
    pub mappings: Option<Usmap>,

    /// Object exports
    pub exports: Vec<Export<Index>>,

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
pub struct ReadExport<Index: PackageIndexTrait> {
    export: Export<Index>,
    new_map_key_overrides: IndexedMap<String, String>,
    new_map_value_overrides: IndexedMap<String, String>,
    new_array_overrides: IndexedMap<String, String>,
}

impl<Index: PackageIndexTrait> ReadExport<Index> {
    /// Create a new `ReadExport` instance
    pub fn new(
        export: Export<Index>,
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
    pub fn reduce(self, asset_data: &mut AssetData<Index>) -> Export<Index> {
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

impl<Index: PackageIndexTrait> AssetData<Index> {
    /// Creates a new `AssetData` instance
    pub fn new() -> AssetData<Index> {
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
        self.summary.custom_versions =
            CustomVersion::get_default_custom_version_container(engine_version);
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
        self.summary
            .custom_versions
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
    pub fn get_export(&self, index: PackageIndex) -> Option<&Export<Index>> {
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
    pub fn get_export_mut(&mut self, index: PackageIndex) -> Option<&mut Export<Index>> {
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
    pub fn get_class_export(&self) -> Option<&ClassExport<Index>> {
        self.exports
            .iter()
            .find_map(|e| cast!(Export, ClassExport, e))
    }

    /// Get if the asset has unversioned properties
    pub fn has_unversioned_properties(&self) -> bool {
        self.summary
            .package_flags
            .contains(EPackageFlags::PKG_UNVERSIONED_PROPERTIES)
    }
}

impl<Index: PackageIndexTrait> Default for AssetData<Index> {
    fn default() -> Self {
        Self {
            use_event_driven_loader: false,
            summary: PackageFileSummary {
                unversioned: true,
                ..Default::default()
            },
            engine_version: EngineVersion::UNKNOWN,
            object_version: ObjectVersion::UNKNOWN,
            object_version_ue5: ObjectVersionUE5::UNKNOWN,
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
pub trait AssetTrait<Index: PackageIndexTrait> {
    /// Gets a reference to the asset data
    fn get_asset_data(&self) -> &AssetData<Index>;
    /// Gets a mutable reference to the asset data
    fn get_asset_data_mut(&mut self) -> &mut AssetData<Index>;

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
pub trait ExportReaderTrait<Index: PackageIndexTrait>:
    ArchiveReader<Index> + AssetTrait<Index> + Sized
{
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
        base_export: BaseExport<Index>,
        next_starting: u64,
    ) -> Result<ReadExport<Index>, Error> {
        self.seek(SeekFrom::Start(base_export.serial_offset as u64))?;

        //todo: manual skips
        let export_class_type = self
            .get_export_class_type(base_export.class_index)
            .ok_or_else(|| Error::invalid_package_index("Unknown class type".to_string()))?;

        let mut new_map_key_overrides = IndexedMap::new();
        let mut new_map_value_overrides = IndexedMap::new();
        let new_array_overrides = IndexedMap::new();

        let mut export: Export<Index> = export_class_type.get_content(|class| {
            Ok::<Export<Index>, Error>(match class {
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
                                                .get_object_name_packageindex(
                                                    struct_property.struct_value,
                                                )
                                                .map(|e| e.get_owned_content()),
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
                                                .get_object_name_packageindex(
                                                    struct_property.struct_value,
                                                )
                                                .map(|e| e.get_owned_content()),
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

            let export: Export<Index> = RawExport::from_base(base_export, self)?.into();
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
    fn read_export(
        &mut self,
        base_export: BaseExport<Index>,
        next_starting: u64,
    ) -> Result<Export<Index>, Error> {
        let serial_offset = base_export.serial_offset as u64;

        match self.read_export_no_raw(base_export.clone(), next_starting) {
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

impl<Index: PackageIndexTrait, R: ArchiveReader<Index> + AssetTrait<Index> + Sized>
    ExportReaderTrait<Index> for R
{
}
