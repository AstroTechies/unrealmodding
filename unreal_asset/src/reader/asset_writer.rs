//! Archive writer

use std::io;

use byteorder::ByteOrder;

use crate::error::Error;
use crate::properties::Property;
use crate::reader::asset_trait::AssetTrait;
use crate::types::{FName, Guid};
use crate::unversioned::header::UnversionedHeader;

/// A trait that allows for writing to an archive in an asset-specific way
pub trait AssetWriter: AssetTrait {
    /// Write a `Guid` property
    fn write_property_guid(&mut self, guid: &Option<Guid>) -> Result<(), Error>;
    /// Write an `FName`
    fn write_fname(&mut self, fname: &FName) -> Result<(), Error>;

    /// Generate an unversioned header for an unversioned package
    fn generate_unversioned_header(
        &mut self,
        properties: &[Property],
        parent_name: &FName,
    ) -> Result<Option<(UnversionedHeader, Vec<Property>)>, Error>;

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
    /// Write an FString
    fn write_fstring(&mut self, value: Option<&str>) -> Result<usize, Error>;
    /// Write all of the bytes in the slice
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
    /// Write `bool`
    fn write_bool(&mut self, value: bool) -> io::Result<()>;
}
