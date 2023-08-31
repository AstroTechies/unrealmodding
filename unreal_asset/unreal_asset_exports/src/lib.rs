#![deny(missing_docs)]
#![allow(non_upper_case_globals)]

//! Unreal asset exports

use std::fmt::Debug;



use unreal_asset_base::{reader::ArchiveWriter, types::PackageIndexTrait, Error, FNameContainer};

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
pub trait ExportNormalTrait<Index: PackageIndexTrait> {
    /// Get a reference to `NormalExport`
    fn get_normal_export(&'_ self) -> Option<&'_ NormalExport<Index>>;
    /// Get a mutable reference to `NormalExport`
    fn get_normal_export_mut(&'_ mut self) -> Option<&'_ mut NormalExport<Index>>;
}

/// This must be implemented for all Exports
pub trait ExportBaseTrait<Index: PackageIndexTrait> {
    /// Get a reference to `BaseExport`
    fn get_base_export(&'_ self) -> &'_ BaseExport<Index>;
    /// Get a mutable reference to `BaseExport`
    fn get_base_export_mut(&'_ mut self) -> &'_ mut BaseExport<Index>;
}

/// Implement `ExportNormalTrait` + `ExportBaseTrait` for an export that contains a `NormalExport`
#[macro_export]
macro_rules! implement_get {
    ($name:ident) => {
        impl<Index: unreal_asset_base::types::PackageIndexTrait> $crate::ExportNormalTrait<Index>
            for $name<Index>
        {
            fn get_normal_export(&self) -> Option<&NormalExport<Index>> {
                Some(&self.normal_export)
            }

            fn get_normal_export_mut(&mut self) -> Option<&mut NormalExport<Index>> {
                Some(&mut self.normal_export)
            }
        }

        impl<Index: unreal_asset_base::types::PackageIndexTrait> $crate::ExportBaseTrait<Index>
            for $name<Index>
        {
            fn get_base_export(&self) -> &BaseExport<Index> {
                &self.normal_export.base_export
            }

            fn get_base_export_mut(&mut self) -> &mut BaseExport<Index> {
                &mut self.normal_export.base_export
            }
        }
    };
}

/// This must be implemented for all Exports
pub trait ExportTrait<Index: PackageIndexTrait>: Debug + Clone + PartialEq + Eq {
    /// Write this `Export` to an asset
    fn write<Writer: ArchiveWriter<Index>>(&self, asset: &mut Writer) -> Result<(), Error>;
}

/// Export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq)]
#[container_nobounds]
pub enum Export<Index: PackageIndexTrait> {
    /// Base export
    BaseExport(BaseExport<Index>),
    /// Class export
    ClassExport(ClassExport<Index>),
    /// Enum export
    EnumExport(EnumExport<Index>),
    /// Level export
    LevelExport(LevelExport<Index>),
    /// Normal export, usually the base for all other exports
    NormalExport(NormalExport<Index>),
    /// Property export
    PropertyExport(PropertyExport<Index>),
    /// Raw export, exists if an export failed to deserialize
    RawExport(RawExport<Index>),
    /// String table export
    StringTableExport(StringTableExport<Index>),
    /// Struct export
    StructExport(StructExport<Index>),
    /// User defined struct export
    UserDefinedStructExport(UserDefinedStructExport<Index>),
    /// Function export
    FunctionExport(FunctionExport<Index>),
    /// Data table export
    DataTableExport(DataTableExport<Index>),
    /// World export
    WorldExport(WorldExport<Index>),
}

/// Macro to mimic `enum_dispatch` functionality because we need generics in traits
macro_rules! manual_dispatch {
    ($($variant:ident),*) => {
        impl<Index: PackageIndexTrait> ExportTrait<Index> for Export<Index> {
            fn write<Writer: ArchiveWriter<Index>>(&self, asset: &mut Writer) -> Result<(), Error> {
                match self {
                    $(
                        Export::$variant(e) => e.write(asset)
                    ),*
                }
            }
        }

        impl<Index: PackageIndexTrait> ExportBaseTrait<Index> for Export<Index> {
            fn get_base_export(&self) -> &BaseExport<Index> {
                match self {
                    $(
                        Export::$variant(e) => e.get_base_export()
                    ),*
                }
            }

            fn get_base_export_mut(&mut self) -> &mut BaseExport<Index> {
                match self {
                    $(
                        Export::$variant(e) => e.get_base_export_mut()
                    ),*
                }
            }
        }

        impl<Index: PackageIndexTrait> ExportNormalTrait<Index> for Export<Index> {
            fn get_normal_export(&self) -> Option<&NormalExport<Index>> {
                match self {
                    $(
                        Export::$variant(e) => e.get_normal_export()
                    ),*
                }
            }

            fn get_normal_export_mut(&mut self) -> Option<&mut NormalExport<Index>> {
                match self {
                    $(
                        Export::$variant(e) => e.get_normal_export_mut()
                    ),*
                }
            }
        }

        $(
            impl<Index: PackageIndexTrait> Into<Export<Index>> for $variant<Index> {
                fn into(self) -> Export<Index> {
                    Export::$variant(self)
                }
            }
        )*
    };
}

manual_dispatch! {
    BaseExport,
    ClassExport,
    EnumExport,
    LevelExport,
    NormalExport,
    PropertyExport,
    RawExport,
    StringTableExport,
    StructExport,
    UserDefinedStructExport,
    FunctionExport,
    DataTableExport,
    WorldExport
}

// todo: impl hash for export
