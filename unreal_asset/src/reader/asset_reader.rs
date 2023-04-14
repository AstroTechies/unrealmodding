//! Archive reader

use std::io;

use byteorder::ByteOrder;

use crate::error::Error;
use crate::reader::asset_trait::AssetTrait;
use crate::types::{FName, Guid};

/// A trait that allows reading from an archive in an asset-specific way
pub trait AssetReader: AssetTrait {
    /// Read a `Guid` property
    fn read_property_guid(&mut self) -> Result<Option<Guid>, Error>;
    /// Read an `FName`
    fn read_fname(&mut self) -> Result<FName, Error>;

    /// Read an array with specified length
    ///
    /// # Examples
    ///
    /// This reads an array of 12 ints
    /// ```no_run,ignore
    /// use unreal_asset::reader::asset_reader::AssetReader;
    /// use byteorder::LittleEndian;
    ///
    /// let reader: AssetReader = ...;
    /// let ints = reader.read_array_with_length(12, |e| e.read_i32::<LittleEndian>()?)?;
    /// ```
    fn read_array_with_length<T>(
        &mut self,
        length: i32,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error>;

    /// Read an array with the length being read from this archive
    ///
    /// This reads an `i32` to determine the archive length, then calls the getter N times
    fn read_array<T>(
        &mut self,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error>;

    /// Read `u8`
    fn read_u8(&mut self) -> io::Result<u8>;
    /// Read `i8`
    fn read_i8(&mut self) -> io::Result<i8>;
    /// Read `u16`
    fn read_u16<T: ByteOrder>(&mut self) -> io::Result<u16>;
    /// Read `i16`
    fn read_i16<T: ByteOrder>(&mut self) -> io::Result<i16>;
    /// Read `u32`
    fn read_u32<T: ByteOrder>(&mut self) -> io::Result<u32>;
    /// Read `i32`
    fn read_i32<T: ByteOrder>(&mut self) -> io::Result<i32>;
    /// Read `u64`
    fn read_u64<T: ByteOrder>(&mut self) -> io::Result<u64>;
    /// Read `i64`
    fn read_i64<T: ByteOrder>(&mut self) -> io::Result<i64>;
    /// Read `f32`
    fn read_f32<T: ByteOrder>(&mut self) -> io::Result<f32>;
    /// Read `f64`
    fn read_f64<T: ByteOrder>(&mut self) -> io::Result<f64>;
    /// Read an FString
    fn read_fstring(&mut self) -> Result<Option<String>, Error>;
    /// Read an exact amount of bytes into a slice
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()>;
    /// Read `bool`
    fn read_bool(&mut self) -> io::Result<bool>;
}
