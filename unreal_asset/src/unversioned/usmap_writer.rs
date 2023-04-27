//! .usmap file writer

use std::io;

use super::usmap_trait::UsmapTrait;

/// .usmap Writer trait
pub trait UsmapWriter: UsmapTrait {
    /// write an `i8` in LE order
    fn write_i8(&mut self, value: i8) -> io::Result<()>;
    /// write an `u8` in LE order
    fn write_u8(&mut self, value: u8) -> io::Result<()>;
    /// write an `i16` in LE order
    fn write_i16(&mut self, value: i16) -> io::Result<()>;
    /// write a `u16` in LE order
    fn write_u16(&mut self, value: u16) -> io::Result<()>;
    /// write an `i32` in LE order
    fn write_i32(&mut self, value: i32) -> io::Result<()>;
    /// write a `u32` in LE order
    fn write_u32(&mut self, value: u32) -> io::Result<()>;
    /// write an `i64` in LE order
    fn write_i64(&mut self, value: i64) -> io::Result<()>;
    /// write a `u64` in LE order
    fn write_u64(&mut self, value: u64) -> io::Result<()>;
    /// write an `f32` in LE order
    fn write_f32(&mut self, value: f32) -> io::Result<()>;
    /// write an `f64` in LE order
    fn write_f64(&mut self, value: f64) -> io::Result<()>;
    /// write an FString
    fn write_fstring(&mut self, value: &str) -> io::Result<usize>;
    /// write a name
    fn write_name(&mut self, name: Option<&str>) -> io::Result<()>;
}
