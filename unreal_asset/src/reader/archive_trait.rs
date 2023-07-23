//! Archive property trait

use std::fmt::Display;
use std::io::{self, SeekFrom};

use crate::asset::name_map::NameMap;
use crate::containers::indexed_map::IndexedMap;
use crate::containers::shared_resource::SharedResource;
use crate::custom_version::{CustomVersion, CustomVersionTrait};
use crate::engine_version::EngineVersion;
use crate::exports::class_export::ClassExport;
use crate::object_version::{ObjectVersion, ObjectVersionUE5};
use crate::types::{fname::FName, PackageIndex};
use crate::unversioned::Usmap;
use crate::{Import, ParentClassInfo};

/// An enum to help identify current archive type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ArchiveType {
    /// Raw archive
    Raw,
    /// Archive used to read .uasset/.uexp files
    UAsset,
    /// Archive used to read .usmap files
    Usmap,
    /// Archive used to read zen files
    Zen,
}

impl Display for ArchiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveType::Raw => write!(f, "Raw"),
            ArchiveType::UAsset => write!(f, "UAsset"),
            ArchiveType::Usmap => write!(f, "Usmap"),
            ArchiveType::Zen => write!(f, "Zen"),
        }
    }
}

/// A trait that allows accessing data about the archive that is currently being read
pub trait ArchiveTrait {
    /// Get archive type
    fn get_archive_type(&self) -> ArchiveType;

    /// Get a custom version from this archive
    ///
    /// # Example
    ///
    /// ```no_run,ignore
    /// use unreal_asset::{
    ///     reader::asset_trait::ArchiveTrait,
    ///     custom_version::FFrameworkObjectVersion,
    /// };
    /// let archive: impl ArchiveTrait = ...;
    /// println!("{:?}", archive.get_custom_version::<FFrameworkObjectVersion>());
    /// ```
    fn get_custom_version<T>(&self) -> CustomVersion
    where
        T: CustomVersionTrait + Into<i32>;

    /// Get if the asset has unversioned properties
    fn has_unversioned_properties(&self) -> bool;

    /// Get if the archive uses the event driven loader
    fn use_event_driven_loader(&self) -> bool;

    /// Archive data length
    fn data_length(&mut self) -> io::Result<u64> {
        let current_position = self.position();
        self.seek(SeekFrom::End(0))?;
        let length = self.position();
        self.seek(SeekFrom::Start(current_position))?;
        Ok(length)
    }
    /// Current archive cursor position
    fn position(&mut self) -> u64;
    /// Set archive cursor position
    fn set_position(&mut self, pos: u64) -> io::Result<()> {
        self.seek(SeekFrom::Start(pos))?;
        Ok(())
    }
    /// Seek
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64>;

    /// Add a string slice to this archive as an `FName`, `FName` number will be 0
    #[inline(always)]
    fn add_fname(&mut self, value: &str) -> FName {
        self.get_name_map().get_mut().add_fname(value)
    }
    /// Add a string slice to this archive as an `FName`
    #[inline(always)]
    fn add_fname_with_number(&mut self, value: &str, number: i32) -> FName {
        self.get_name_map()
            .get_mut()
            .add_fname_with_number(value, number)
    }

    /// Get FName name map
    fn get_name_map(&self) -> SharedResource<NameMap>;
    /// Get FName name reference by name map index and do something with it
    fn get_name_reference<T>(&self, index: i32, func: impl FnOnce(&str) -> T) -> T {
        func(self.get_name_map().get_ref().get_name_reference(index))
    }

    /// Get struct overrides for an `ArrayProperty`
    fn get_array_struct_type_override(&self) -> &IndexedMap<String, String>;
    /// Get map key overrides for a `MapProperty`
    fn get_map_key_override(&self) -> &IndexedMap<String, String>;
    /// Get map value overrides for a `MapProperty`
    fn get_map_value_override(&self) -> &IndexedMap<String, String>;

    /// Get archive's optional parent class
    fn get_parent_class(&self) -> Option<ParentClassInfo> {
        let class_export = self.get_class_export()?;

        let parent_class_import = self.get_import(class_export.struct_export.super_struct)?;
        let outer_parent_import = self.get_import(parent_class_import.outer_index)?;

        Some(ParentClassInfo {
            parent_class_path: parent_class_import.object_name,
            parent_class_export_name: outer_parent_import.object_name,
        })
    }

    /// Get archive's engine version
    fn get_engine_version(&self) -> EngineVersion;
    /// Get archive's object version
    fn get_object_version(&self) -> ObjectVersion;
    /// Get archive's UE5 object version
    fn get_object_version_ue5(&self) -> ObjectVersionUE5;

    /// Get .usmap mappings
    fn get_mappings(&self) -> Option<&Usmap>;

    /// Searches for and returns this asset's CLassExport, if one exists
    fn get_class_export(&self) -> Option<&ClassExport>;
    /// Get an import by a `PackageIndex`
    fn get_import(&self, index: PackageIndex) -> Option<Import>;
    /// Get export class type by a `PackageIndex`
    fn get_export_class_type(&self, index: PackageIndex) -> Option<FName> {
        match index.is_import() {
            true => self.get_import(index).map(|e| e.object_name),
            false => Some(FName::new_dummy(index.index.to_string(), 0)),
        }
    }
}
