use std::io::{Cursor, Read, SeekFrom, Seek, ErrorKind};
use byteorder::{ReadBytesExt, LittleEndian};

use super::{unreal_types::Guid, types::Vector};
use super::error::Error;

pub trait CursorExt {
    fn read_string(&mut self) -> Result<String, Error>;
    fn read_bool(&mut self) -> Result<bool, Error>;
    fn read_vector(&mut self) -> Result<Vector<f32>, Error>;
    fn read_int_vector(&mut self) -> Result<Vector<i32>, Error>;
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
}