use std::io;

use byteorder::ByteOrder;

use crate::error::Error;
use crate::reader::asset_trait::AssetTrait;
use crate::types::{FName, Guid};

/// A trait that allows reading from an archive in an asset-specific way
pub trait AssetReader: AssetTrait {
    fn read_property_guid(&mut self) -> Result<Option<Guid>, Error>;
    fn read_fname(&mut self) -> Result<FName, Error>;

    fn read_array_with_length<T>(
        &mut self,
        length: i32,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error>;

    fn read_array<T>(
        &mut self,
        getter: impl Fn(&mut Self) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error>;

    fn read_u8(&mut self) -> io::Result<u8>;
    fn read_i8(&mut self) -> io::Result<i8>;
    fn read_u16<T: ByteOrder>(&mut self) -> io::Result<u16>;
    fn read_i16<T: ByteOrder>(&mut self) -> io::Result<i16>;
    fn read_u32<T: ByteOrder>(&mut self) -> io::Result<u32>;
    fn read_i32<T: ByteOrder>(&mut self) -> io::Result<i32>;
    fn read_u64<T: ByteOrder>(&mut self) -> io::Result<u64>;
    fn read_i64<T: ByteOrder>(&mut self) -> io::Result<i64>;
    fn read_f32<T: ByteOrder>(&mut self) -> io::Result<f32>;
    fn read_f64<T: ByteOrder>(&mut self) -> io::Result<f64>;
    fn read_fstring(&mut self) -> io::Result<Option<String>>;
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()>;
    fn read_bool(&mut self) -> io::Result<bool>;
}
