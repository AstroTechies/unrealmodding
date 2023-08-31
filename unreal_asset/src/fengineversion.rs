//! [`FEngineVersion`] type

use std::fmt::Debug;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::{
    reader::{ArchiveReader, ArchiveWriter},
    types::PackageIndexTrait,
    Error,
};

/// EngineVersion for an Asset
#[derive(Debug, Clone)]
pub struct FEngineVersion {
    pub(crate) major: u16,
    pub(crate) minor: u16,
    pub(crate) patch: u16,
    pub(crate) build: u32,
    pub(crate) branch: Option<String>,
}
impl FEngineVersion {
    pub(crate) fn new(
        major: u16,
        minor: u16,
        patch: u16,
        build: u32,
        branch: Option<String>,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            build,
            branch,
        }
    }

    pub(crate) fn read<Reader: ArchiveReader<impl PackageIndexTrait>>(
        cursor: &mut Reader,
    ) -> Result<Self, Error> {
        let major = cursor.read_u16::<LE>()?;
        let minor = cursor.read_u16::<LE>()?;
        let patch = cursor.read_u16::<LE>()?;
        let build = cursor.read_u32::<LE>()?;
        let branch = cursor.read_fstring()?;

        Ok(Self::new(major, minor, patch, build, branch))
    }

    pub(crate) fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        cursor: &mut Writer,
    ) -> Result<(), Error> {
        cursor.write_u16::<LE>(self.major)?;
        cursor.write_u16::<LE>(self.minor)?;
        cursor.write_u16::<LE>(self.patch)?;
        cursor.write_u32::<LE>(self.build)?;
        cursor.write_fstring(self.branch.as_deref())?;
        Ok(())
    }

    pub(crate) fn unknown() -> Self {
        Self::new(0, 0, 0, 0, None)
    }
}
