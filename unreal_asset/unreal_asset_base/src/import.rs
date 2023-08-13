//! Asset Import struct

use crate::FNameContainer;

use crate::types::{FName, PackageIndex};

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

// silly `FNameContainer` fix
mod unreal_asset_base {
    pub use crate::types;
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
