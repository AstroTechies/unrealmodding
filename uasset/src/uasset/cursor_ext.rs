use std::io::{Cursor, Read, SeekFrom, Seek, Write};
use std::mem::size_of;
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};

use super::{types::Vector};
use super::error::Error;

pub trait CursorExt {
    fn read_string(&mut self) -> Result<String, Error>;
    fn read_bool(&mut self) -> Result<bool, Error>;
    fn read_vector(&mut self) -> Result<Vector<f32>, Error>;
    fn read_int_vector(&mut self) -> Result<Vector<i32>, Error>;

    fn write_string(&mut self, string: &String) -> Result<usize, Error>;
    fn write_bool(&mut self, value: bool) -> Result<(), Error>;
}

impl CursorExt for Cursor<Vec<u8>> {
    // read string of format <length u32><string><null>
    fn read_string(&mut self) -> Result<String, Error> { // todo: unicode encoding
        let len = self.read_u32::<LittleEndian>()?;

        if len == 0 {
            return Ok(String::new());
        }

        let mut buf = vec![0u8; len as usize - 1];
        self.read_exact(&mut buf)?;
        self.seek(SeekFrom::Current(1))?;
        Ok(String::from_utf8(buf).unwrap_or(String::from("None")))
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        let res = self.read_u8()?;
        Ok(res > 0)
    }

    fn read_vector(&mut self) -> Result<Vector<f32>, Error> {
        Ok(
            Vector::new(
                self.read_f32::<LittleEndian>()?,
                self.read_f32::<LittleEndian>()?,
                self.read_f32::<LittleEndian>()?
            )
        )
    }

    fn read_int_vector(&mut self) -> Result<Vector<i32>, Error> {
        Ok(
            Vector::new(
                self.read_i32::<LittleEndian>()?,
                self.read_i32::<LittleEndian>()?,
                self.read_i32::<LittleEndian>()?
            )
        )
    }

    fn write_string(&mut self, string: &String) -> Result<usize, Error> {
        let is_unicode = string.len() != string.chars().count();
        match is_unicode {
            true => self.write_i32::<LittleEndian>(-(string.len() as i32) + 1)?,
            false => self.write_i32::<LittleEndian>(string.len() as i32 + 1)?
        };
        let bytes = string.clone().into_bytes();
        self.write(&bytes)?;
        self.write(&[0u8; 1])?;
        Ok(bytes.len())
    }

    fn write_bool(&mut self, value: bool) -> Result<(), Error> {
        self.write_u8(match value {
            true => 1,
            false => 0
        })?;
        Ok(())
    }
}