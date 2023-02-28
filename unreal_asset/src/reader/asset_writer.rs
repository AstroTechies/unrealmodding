use std::io;

use byteorder::ByteOrder;

use crate::error::Error;
use crate::reader::asset_trait::AssetTrait;
use crate::types::{FName, Guid};

/// A trait that allows for writing to an archive in an asset-specific way
pub trait AssetWriter: AssetTrait {
    fn write_property_guid(&mut self, guid: &Option<Guid>) -> Result<(), Error>;
    fn write_fname(&mut self, fname: &FName) -> Result<(), Error>;

    fn write_u8(&mut self, value: u8) -> io::Result<()>;
    fn write_i8(&mut self, value: i8) -> io::Result<()>;
    fn write_u16<T: ByteOrder>(&mut self, value: u16) -> io::Result<()>;
    fn write_i16<T: ByteOrder>(&mut self, value: i16) -> io::Result<()>;
    fn write_u32<T: ByteOrder>(&mut self, value: u32) -> io::Result<()>;
    fn write_i32<T: ByteOrder>(&mut self, value: i32) -> io::Result<()>;
    fn write_u64<T: ByteOrder>(&mut self, value: u64) -> io::Result<()>;
    fn write_i64<T: ByteOrder>(&mut self, value: i64) -> io::Result<()>;
    fn write_f32<T: ByteOrder>(&mut self, value: f32) -> io::Result<()>;
    fn write_f64<T: ByteOrder>(&mut self, value: f64) -> io::Result<()>;
    fn write_fstring(&mut self, value: Option<&str>) -> io::Result<usize>;
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
    fn write_bool(&mut self, value: bool) -> io::Result<()>;
}
