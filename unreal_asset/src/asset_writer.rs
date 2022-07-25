use std::io::{self, Cursor};

use byteorder::ByteOrder;

use crate::{
    asset_trait::AssetTrait,
    error::Error,
    unreal_types::{FName, Guid},
};

pub trait AssetWriter: AssetTrait {
    fn write_property_guid(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        guid: &Option<Guid>,
    ) -> Result<(), Error>;
    fn write_fname(&self, cursor: &mut Cursor<Vec<u8>>, fname: &FName) -> Result<(), Error>;

    fn write_u8(&self, cursor: &mut Cursor<Vec<u8>>, value: u8) -> Result<(), io::Error>;
    fn write_i8(&self, cursor: &mut Cursor<Vec<u8>>, value: i8) -> Result<(), io::Error>;
    fn write_u16<T: ByteOrder>(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: u16,
    ) -> Result<(), io::Error>;
    fn write_i16<T: ByteOrder>(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: i16,
    ) -> Result<(), io::Error>;
    fn write_u32<T: ByteOrder>(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: u32,
    ) -> Result<(), io::Error>;
    fn write_i32<T: ByteOrder>(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: i32,
    ) -> Result<(), io::Error>;
    fn write_u64<T: ByteOrder>(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: u64,
    ) -> Result<(), io::Error>;
    fn write_i64<T: ByteOrder>(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: i64,
    ) -> Result<(), io::Error>;
    fn write_f32<T: ByteOrder>(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: f32,
    ) -> Result<(), io::Error>;
    fn write_f64<T: ByteOrder>(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: f64,
    ) -> Result<(), io::Error>;
    fn write_string(
        &self,
        cursor: &mut Cursor<Vec<u8>>,
        value: &Option<String>,
    ) -> Result<usize, Error>;
    fn write_all(&self, cursor: &mut Cursor<Vec<u8>>, buf: &[u8]) -> Result<(), io::Error>;
    fn write_bool(&self, cursor: &mut Cursor<Vec<u8>>, value: bool) -> Result<(), Error>;
}
