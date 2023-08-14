//! Archive writer

use std::io::{self, Write};

use byteorder::{WriteBytesExt, LE};

use crate::error::{Error, FNameError};
use crate::object_version::ObjectVersion;
use crate::reader::ArchiveTrait;
use crate::types::FName;
use crate::Guid;

/// A trait that allows for writing to an archive in an asset-specific way
pub trait ArchiveWriter: ArchiveTrait + Write {
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

    /// Write an FString
    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error>;
    /// Write a guid.
    fn write_guid(&mut self, guid: &Guid) -> io::Result<()>;
    /// Write `bool`
    fn write_bool(&mut self, value: bool) -> io::Result<()>;
}

/// A trait that allows for quick implementation of [`ArchiveWriter`] as a passthrough trait for the underlying archive
pub trait PassthroughArchiveWriter: ArchiveTrait + Write {
    /// Passthrough archive writer type
    type Passthrough: ArchiveWriter;
    /// Get the passthrough archive writer
    fn get_passthrough(&mut self) -> &mut Self::Passthrough;
}

impl<Writer, Passthrough> ArchiveWriter for Passthrough
where
    Writer: ArchiveWriter,
    Passthrough: PassthroughArchiveWriter<Passthrough = Writer>,
{
    #[inline(always)]
    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error> {
        self.get_passthrough().write_fstring(value)
    }

    #[inline(always)]
    fn write_guid(&mut self, guid: &Guid) -> io::Result<()> {
        self.get_passthrough().write_guid(guid)
    }

    #[inline(always)]
    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.get_passthrough().write_bool(value)
    }
}
