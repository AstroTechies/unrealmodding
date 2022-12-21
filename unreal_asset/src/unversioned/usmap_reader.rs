use std::io;

use crate::error::Error;

pub trait UsmapReader {
    fn read_i8(&mut self) -> Result<i8, io::Error>;
    fn read_u8(&mut self) -> Result<u8, io::Error>;
    fn read_i16(&mut self) -> Result<i16, io::Error>;
    fn read_u16(&mut self) -> Result<u16, io::Error>;
    fn read_i32(&mut self) -> Result<i32, io::Error>;
    fn read_u32(&mut self) -> Result<u32, io::Error>;
    fn read_i64(&mut self) -> Result<i64, io::Error>;
    fn read_u64(&mut self) -> Result<u64, io::Error>;
    fn read_f32(&mut self) -> Result<f32, io::Error>;
    fn read_f64(&mut self) -> Result<f64, io::Error>;
    fn read_string(&mut self) -> Result<String, Error>;
    fn read_name(&mut self) -> Result<String, io::Error>;
}
