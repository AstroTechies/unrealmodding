//! .usmap file reader

use std::io;

use crate::error::Error;

use super::usmap_trait::UsmapTrait;

/// .usmap Reader trait
pub trait UsmapReader: UsmapTrait {
    /// read an `i8` in LE order
    fn read_i8(&mut self) -> io::Result<i8>;
    /// read an `u8` in LE order
    fn read_u8(&mut self) -> io::Result<u8>;
    /// read an `i16` in LE order
    fn read_i16(&mut self) -> io::Result<i16>;
    /// read a `u16` in LE order
    fn read_u16(&mut self) -> io::Result<u16>;
    /// read an `i32` in LE order
    fn read_i32(&mut self) -> io::Result<i32>;
    /// read a `u32` in LE order
    fn read_u32(&mut self) -> io::Result<u32>;
    /// read an `i64` in LE order
    fn read_i64(&mut self) -> io::Result<i64>;
    /// read a `u64` in LE order
    fn read_u64(&mut self) -> io::Result<u64>;
    /// read an `f32` in LE order
    fn read_f32(&mut self) -> io::Result<f32>;
    /// read an `f64` in LE order
    fn read_f64(&mut self) -> io::Result<f64>;
    /// read an FString
    fn read_fstring(&mut self) -> Result<Option<String>, Error>;
    /// read a name
    fn read_name(&mut self) -> io::Result<Option<String>>;
}
