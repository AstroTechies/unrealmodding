use std::io::{self};

use byteorder::ByteOrder;

use crate::{
    error::Error,
    unreal_types::{FName, Guid},
};

use super::asset_trait::AssetTrait;

pub trait AssetWriter: AssetTrait {
    fn write_property_guid(&mut self, guid: &Option<Guid>) -> Result<(), Error>;
    fn write_fname(&mut self, fname: &FName) -> Result<(), Error>;

    fn write_u8(&mut self, value: u8) -> Result<(), io::Error>;
    fn write_i8(&mut self, value: i8) -> Result<(), io::Error>;
    fn write_u16<T: ByteOrder>(&mut self, value: u16) -> Result<(), io::Error>;
    fn write_i16<T: ByteOrder>(&mut self, value: i16) -> Result<(), io::Error>;
    fn write_u32<T: ByteOrder>(&mut self, value: u32) -> Result<(), io::Error>;
    fn write_i32<T: ByteOrder>(&mut self, value: i32) -> Result<(), io::Error>;
    fn write_u64<T: ByteOrder>(&mut self, value: u64) -> Result<(), io::Error>;
    fn write_i64<T: ByteOrder>(&mut self, value: i64) -> Result<(), io::Error>;
    fn write_f32<T: ByteOrder>(&mut self, value: f32) -> Result<(), io::Error>;
    fn write_f64<T: ByteOrder>(&mut self, value: f64) -> Result<(), io::Error>;
    fn write_string(&mut self, value: &Option<String>) -> Result<usize, Error>;
    fn write_all(&mut self, buf: &[u8]) -> Result<(), io::Error>;
    fn write_bool(&mut self, value: bool) -> Result<(), Error>;
}
