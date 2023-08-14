//! Archive writer

use std::io::{self, Write};

use byteorder::{WriteBytesExt, LE};

use crate::custom_version::CustomVersion;
use crate::enums::ECustomVersionSerializationFormat;
use crate::error::{Error, FNameError};
use crate::object_version::ObjectVersion;
use crate::reader::ArchiveTrait;
use crate::types::{FName, PackageIndexTrait};
use crate::Guid;

/// A trait that allows for writing to an archive in an asset-specific way
pub trait ArchiveWriter<Index: PackageIndexTrait>: ArchiveTrait<Index> + Write {
    /// Write a `Guid` property
    fn write_property_guid(&mut self, guid: Option<&Guid>) -> Result<(), Error> {
        if self.get_object_version() >= ObjectVersion::VER_UE4_PROPERTY_GUID_IN_PROPERTY_TAG {
            self.write_bool(guid.is_some())?;
            if let Some(data) = guid {
                self.write_guid(data)?;
            }
        }

        Ok(())
    }
    /// Write an `FName`
    fn write_fname(&mut self, fname: &FName) -> Result<(), Error> {
        match fname {
            FName::Backed {
                index,
                number,
                ty: _,
                name_map: _,
            } => {
                self.write_i32::<LE>(*index)?;
                self.write_i32::<LE>(*number)?;
                Ok(())
            }
            FName::Dummy { value, number } => {
                Err(FNameError::dummy_serialize(value, *number).into())
            }
        }
    }

    /// Write custom version container
    fn write_custom_version_container(
        &mut self,
        format: ECustomVersionSerializationFormat,
        container: &[CustomVersion],
    ) -> Result<(), Error> {
        match format {
            ECustomVersionSerializationFormat::Unknown => {
                return Err(Error::invalid_file(String::from(
                    "Cannot read a custom version container with an unknown serialization format",
                )))
            }
            ECustomVersionSerializationFormat::Enums => {
                return Err(Error::unimplemented(String::from(
                    "Custom version container with Enums serialization format is unimplemented",
                )))
            }
            _ => {}
        }

        for version in container {
            self.write_guid(&version.guid)?;
            self.write_i32::<LE>(version.version)?;
        }

        Ok(())
    }

    /// Write an FString
    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error>;
    /// Write a guid.
    fn write_guid(&mut self, guid: &Guid) -> io::Result<()>;
    /// Write `bool`
    fn write_bool(&mut self, value: bool) -> io::Result<()>;
}

/// A macro that allows for quick implementation of [`ArchiveWriter`] as a passthrough for the underlying archive
#[macro_export]
macro_rules! passthrough_archive_writer {
    ($passthrough:ident) => {
        #[inline(always)]
        fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error> {
            self.$passthrough.write_fstring(value)
        }

        #[inline(always)]
        fn write_guid(&mut self, guid: &unreal_helpers::Guid) -> std::io::Result<()> {
            self.$passthrough.write_guid(guid)
        }

        #[inline(always)]
        fn write_bool(&mut self, value: bool) -> std::io::Result<()> {
            self.$passthrough.write_bool(value)
        }
    };
}
