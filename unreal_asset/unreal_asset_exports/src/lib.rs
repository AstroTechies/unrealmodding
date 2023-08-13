#![deny(missing_docs)]
#![allow(non_upper_case_globals)]

//! Unreal asset exports

use std::fmt::Debug;

use enum_dispatch::enum_dispatch;

use unreal_asset_base::{reader::ArchiveWriter, Error, FNameContainer};

pub mod properties;

pub mod base_export;
pub mod class_export;
pub mod data_table_export;
pub mod enum_export;
pub mod function_export;
pub mod level_export;
pub mod normal_export;
pub mod property_export;
pub mod raw_export;
pub mod string_table_export;
pub mod struct_export;
pub mod user_defined_struct_export;
pub mod world_export;

pub use self::{
    base_export::BaseExport, class_export::ClassExport, data_table_export::DataTableExport,
    enum_export::EnumExport, function_export::FunctionExport, level_export::LevelExport,
    normal_export::NormalExport, property_export::PropertyExport, raw_export::RawExport,
    string_table_export::StringTableExport, struct_export::StructExport,
    user_defined_struct_export::UserDefinedStructExport, world_export::WorldExport,
};

/// This must be implemented for all Exports
/// Allows for getting a NormalExport from any export containing one
/// If an export doesn't have one return None
#[enum_dispatch]
pub trait ExportNormalTrait {
    /// Get a reference to `NormalExport`
    fn get_normal_export(&'_ self) -> Option<&'_ NormalExport>;
    /// Get a mutable reference to `NormalExport`
    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut NormalExport>;
}

/// This must be implemented for all Exports
#[enum_dispatch]
pub trait ExportBaseTrait {
    /// Get a reference to `BaseExport`
    fn get_base_export(&'_ self) -> &'_ BaseExport;
    /// Get a mutable reference to `BaseExport`
    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport;
}

/// Implement `ExportNormalTrait` + `ExportBaseTrait` for an export that contains a `NormalExport`
#[macro_export]
macro_rules! implement_get {
    ($name:ident) => {
        impl $crate::ExportNormalTrait for $name {
            fn get_normal_export(&self) -> Option<&NormalExport> {
                Some(&self.normal_export)
            }

            fn get_normal_export_mut(&mut self) -> Option<&mut NormalExport> {
                Some(&mut self.normal_export)
            }
        }

        impl $crate::ExportBaseTrait for $name {
            fn get_base_export(&self) -> &BaseExport {
                &self.normal_export.base_export
            }

            fn get_base_export_mut(&mut self) -> &mut BaseExport {
                &mut self.normal_export.base_export
            }
        }
    };
}

/// This must be implemented for all Exports
#[enum_dispatch]
pub trait ExportTrait: Debug + Clone + PartialEq + Eq {
    /// Write this `Export` to an asset
    fn write<Writer: ArchiveWriter>(&self, asset: &mut Writer) -> Result<(), Error>;
}

/// Export
#[enum_dispatch(ExportTrait, ExportNormalTrait, ExportBaseTrait)]
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq)]
#[container_nobounds]
pub enum Export {
    /// Base export
    BaseExport,
    /// Class export
    ClassExport,
    /// Enum export
    EnumExport,
    /// Level export
    LevelExport,
    /// Normal export, usually the base for all other exports
    NormalExport,
    /// Property export
    PropertyExport,
    /// Raw export, exists if an export failed to deserialize
    RawExport,
    /// String table export
    StringTableExport,
    /// Struct export
    StructExport,
    /// User defined struct export
    UserDefinedStructExport,
    /// Function export
    FunctionExport,
    /// Data table export
    DataTableExport,
    /// World export
    WorldExport,
}

// todo: impl hash for export
