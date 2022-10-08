use std::io;

use byteorder::ByteOrder;

use crate::error::Error;
use crate::reader::asset_trait::AssetTrait;
use crate::unreal_types::{FName, Guid};

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

    fn read_u8(&mut self) -> Result<u8, io::Error>;
    fn read_i8(&mut self) -> Result<i8, io::Error>;
    fn read_u16<T: ByteOrder>(&mut self) -> Result<u16, io::Error>;
    fn read_i16<T: ByteOrder>(&mut self) -> Result<i16, io::Error>;
    fn read_u32<T: ByteOrder>(&mut self) -> Result<u32, io::Error>;
    fn read_i32<T: ByteOrder>(&mut self) -> Result<i32, io::Error>;
    fn read_u64<T: ByteOrder>(&mut self) -> Result<u64, io::Error>;
    fn read_i64<T: ByteOrder>(&mut self) -> Result<i64, io::Error>;
    fn read_f32<T: ByteOrder>(&mut self) -> Result<f32, io::Error>;
    fn read_f64<T: ByteOrder>(&mut self) -> Result<f64, io::Error>;
    fn read_string(&mut self) -> Result<Option<String>, Error>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), io::Error>;
    fn read_bool(&mut self) -> Result<bool, Error>;
}
