//! Archive property trait

use std::io::{self, SeekFrom};

use crate::containers::indexed_map::IndexedMap;
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::EngineVersion;
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::types::{FName, PackageIndex};
use crate::unversioned::Usmap;
use crate::{Import, ParentClassInfo};

/// A trait that allows accessing data about the archive that is currently being read
pub trait AssetTrait {
    /// Get a custom version from this archive
    ///
    /// # Example
    ///
    /// ```no_run,ignore
    /// use unreal_asset::{
    ///     reader::asset_trait::AssetTrait,
    ///     custom_version::FFrameworkObjectVersion,
    /// };
    /// let archive: impl AssetTrait = ...;
    /// println!("{:?}", archive.get_custom_version::<FFrameworkObjectVersion>());
    /// ```
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>;

    /// Current archive cursor position
    fn position(&mut self) -> u64;
    /// Set archive cursor position
    fn set_position(&mut self, pos: u64);
    /// Seek
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64>;

    /// Add a string slice to this archive as an `FName`, `FName` number will be 0
    fn add_fname(&mut self, value: &str) -> FName;
    /// Add a string slice to this archive as an `FName`
    fn add_fname_with_number(&mut self, value: &str, number: i32) -> FName;

    /// Get FName name map index list
    fn get_name_map_index_list(&self) -> &[String];
    /// Get FName name reference by name map index
    fn get_name_reference(&self, index: i32) -> String;

    /// Get struct overrides for an `ArrayProperty`
    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String>;
    /// Get map key overrides for a `MapProperty`
    fn get_map_key_override(&self) -> &IndexedMap<String, String>;
    /// Get map value overrides for a `MapProperty`
    fn get_map_value_override(&self) -> &IndexedMap<String, String>;

    /// Get archive's optional parent class
    fn get_parent_class(&self) -> Option<ParentClassInfo>;
    /// Get archive's optional parent class and cache it
    fn get_parent_class_cached(&mut self) -> Option<&ParentClassInfo>;

    /// Get archive's engine version
    fn get_engine_version(&self) -> EngineVersion;
    /// Get archive's object version
    fn get_object_version(&self) -> ObjectVersion;
    /// Get archive's UE5 object version
    fn get_object_version_ue5(&self) -> ObjectVersionUE5;

    /// Get .usmap mappings
    fn get_mappings(&self) -> Option<&Usmap>;

    /// Get an import by a `PackageIndex`
    fn get_import(&self, index: PackageIndex) -> Option<&Import>;
    /// Get export class type by a `PackageIndex`
    fn get_export_class_type(&self, index: PackageIndex) -> Option<FName>;
}
