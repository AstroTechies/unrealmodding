#![deny(missing_docs)]
//! unreal_asset crate base members

pub mod compression;
pub mod containers;
pub mod crc;
pub mod custom_version;
pub mod engine_version;
pub mod enums;
pub mod error;
pub mod flags;
pub mod fproperty;
pub mod object_version;
pub mod reader;
pub mod types;
pub mod unversioned;
pub mod uproperty;

use types::{fname::FName, PackageIndex};
use unreal_asset_proc_macro::FNameContainer;
pub use unreal_helpers::Guid;

/// Cast a Property/Export to a more specific type
///
/// # Examples
///
/// ```no_run,ignore
/// use unreal_asset::{
///     cast,
///     properties::{
///         Property,
///         int_property::DoubleProperty,
///     },
/// };
/// let a: Property = ...;
/// let b: &DoubleProperty = cast!(Property, DoubleProperty, &a).unwrap();
/// ```
#[macro_export]
macro_rules! cast {
    ($namespace:ident, $type:ident, $field:expr) => {
        match $field {
            $namespace::$type(e) => Some(e),
            _ => None,
        }
    };
}

/// Import struct for an Asset
///
/// This is used for referencing other assets
#[derive(FNameContainer, Debug, Clone, Eq, PartialEq)]
pub struct Import {
    /// Class package
    pub class_package: FName,
    /// Class name
    pub class_name: FName,
    /// Outer index
    #[container_ignore]
    pub outer_index: PackageIndex,
    /// Object name
    pub object_name: FName,
    /// Is the import optional
    pub optional: bool,
}

impl Import {
    /// Create a new `Import` instance
    pub fn new(
        class_package: FName,
        class_name: FName,
        outer_index: PackageIndex,
        object_name: FName,
        optional: bool,
    ) -> Self {
        Import {
            class_package,
            class_name,
            object_name,
            outer_index,
            optional,
        }
    }
}
