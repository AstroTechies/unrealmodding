//! Archive writer

use std::io;

use byteorder::{ByteOrder, LE};

use crate::error::{Error, FNameError};
use crate::object_version::ObjectVersion;
use crate::reader::ArchiveTrait;
use crate::types::FName;
use crate::Guid;

/// A trait that allows for writing to an archive in an asset-specific way
pub trait ArchiveWriter: ArchiveTrait {
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

    /// Write `u8`
    fn write_u8(&mut self, value: u8) -> io::Result<()>;
    /// Write `i8`
    fn write_i8(&mut self, value: i8) -> io::Result<()>;
    /// Write `u16`
    fn write_u16<T: ByteOrder>(&mut self, value: u16) -> io::Result<()>;
    /// Write `i16`
    fn write_i16<T: ByteOrder>(&mut self, value: i16) -> io::Result<()>;
    /// Write `u32`
    fn write_u32<T: ByteOrder>(&mut self, value: u32) -> io::Result<()>;
    /// Write `i32`
    fn write_i32<T: ByteOrder>(&mut self, value: i32) -> io::Result<()>;
    /// Write `u64`
    fn write_u64<T: ByteOrder>(&mut self, value: u64) -> io::Result<()>;
    /// Write `i64`
    fn write_i64<T: ByteOrder>(&mut self, value: i64) -> io::Result<()>;
    /// Write `f32`
    fn write_f32<T: ByteOrder>(&mut self, value: f32) -> io::Result<()>;
    /// Write `f64`
    fn write_f64<T: ByteOrder>(&mut self, value: f64) -> io::Result<()>;
    /// Write all of the bytes in the slice
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
    /// Write an FString
    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error>;
    /// Write a guid.
    fn write_guid(&mut self, guid: &Guid) -> io::Result<()>;
    /// Write `bool`
    fn write_bool(&mut self, value: bool) -> io::Result<()>;
}

/// A trait that allows for quick implementation of [`ArchiveWriter`] as a pastthrough trait for the underlying archive
pub trait PassthroughArchiveWriter: ArchiveTrait {
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
    fn write_u8(&mut self, value: u8) -> io::Result<()> {
        self.get_passthrough().write_u8(value)
    }

    #[inline(always)]
    fn write_i8(&mut self, value: i8) -> io::Result<()> {
        self.get_passthrough().write_i8(value)
    }

    #[inline(always)]
    fn write_u16<T: ByteOrder>(&mut self, value: u16) -> io::Result<()> {
        self.get_passthrough().write_u16::<T>(value)
    }

    #[inline(always)]
    fn write_i16<T: ByteOrder>(&mut self, value: i16) -> io::Result<()> {
        self.get_passthrough().write_i16::<T>(value)
    }

    #[inline(always)]
    fn write_u32<T: ByteOrder>(&mut self, value: u32) -> io::Result<()> {
        self.get_passthrough().write_u32::<T>(value)
    }

    #[inline(always)]
    fn write_i32<T: ByteOrder>(&mut self, value: i32) -> io::Result<()> {
        self.get_passthrough().write_i32::<T>(value)
    }

    #[inline(always)]
    fn write_u64<T: ByteOrder>(&mut self, value: u64) -> io::Result<()> {
        self.get_passthrough().write_u64::<T>(value)
    }

    #[inline(always)]
    fn write_i64<T: ByteOrder>(&mut self, value: i64) -> io::Result<()> {
        self.get_passthrough().write_i64::<T>(value)
    }

    #[inline(always)]
    fn write_f32<T: ByteOrder>(&mut self, value: f32) -> io::Result<()> {
        self.get_passthrough().write_f32::<T>(value)
    }

    #[inline(always)]
    fn write_f64<T: ByteOrder>(&mut self, value: f64) -> io::Result<()> {
        self.get_passthrough().write_f64::<T>(value)
    }

    #[inline(always)]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.get_passthrough().write_all(buf)
    }

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
