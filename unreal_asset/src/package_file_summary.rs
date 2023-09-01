//! Package(asset) file summary

use unreal_asset_base::{
    custom_version::{CustomVersion, CustomVersionTrait},
    flags::EPackageFlags,
};

/// Package file summary
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageFileSummary {
    /// Package flags
    pub package_flags: EPackageFlags,
    /// Export count
    pub export_count: i32,
    /// Import count
    pub import_count: i32,
    /// File licensee version
    pub file_licensee_version: i32,
    /// Custom versions
    pub custom_versions: Vec<CustomVersion>,
    /// Is unversioned
    pub unversioned: bool,
}

impl PackageFileSummary {
    /// Get custom version from
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
}
