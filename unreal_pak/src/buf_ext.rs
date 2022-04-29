use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub trait BufReaderExt {
    fn read_string(&mut self) -> Result<Option<String>, std::io::Error>;
}
pub trait BufWriterExt {
    fn write_string(&mut self, string: Option<&str>) -> Result<(), std::io::Error>;
}

impl<R> BufReaderExt for R
where
    R: Read + Seek,
{
    fn read_string(&mut self) -> Result<Option<String>, std::io::Error> {
        // todo: unicode encoding
        let len = self.read_u32::<LittleEndian>()?;

        if len == 0 {
            return Ok(None);
        }

        let mut buf = vec![0u8; len as usize - 1];
        self.read_exact(&mut buf)?;
        self.seek(SeekFrom::Current(1))?;
        Ok(Some(String::from_utf8(buf).unwrap_or(String::from("None"))))
    }
}

impl<W> BufWriterExt for W
where
    W: Write + Seek,
{
    fn write_string(&mut self, string: Option<&str>) -> Result<(), std::io::Error> {
        if string.is_none() {
            self.write_i32::<LittleEndian>(0)?;
            return Ok(());
        }
        let string = string.clone().unwrap();
        let is_unicode = string.len() != string.chars().count();
        match is_unicode {
            true => self.write_i32::<LittleEndian>(-(string.len() as i32) + 1)?,
            false => self.write_i32::<LittleEndian>(string.len() as i32 + 1)?,
        };
        let bytes = string.clone().as_bytes();
        self.write(&bytes)?;
        self.write(&[0u8; 1])?;
        Ok(())
    }
}
