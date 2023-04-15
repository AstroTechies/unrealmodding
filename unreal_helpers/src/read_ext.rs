//! Extension for anything that implements Read to more easily read Unreal data formats.

use std::io::{self, Read};
use std::mem::size_of;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::FStringError;

/// Extension for anything that implements Read to more easily read Unreal data formats.
pub trait UnrealReadExt {
    /// Read string of format \<length i32\>\<string\>\<null\>
    fn read_fstring(&mut self) -> Result<Option<String>, FStringError>;
    /// Read u8 as bool
    fn read_bool(&mut self) -> io::Result<bool>;
}

impl<R: Read> UnrealReadExt for R {
    fn read_fstring(&mut self) -> Result<Option<String>, FStringError> {
        let len = self.read_i32::<LittleEndian>()?;

        if !(-131072..=131072).contains(&len) {
            return Err(FStringError::InvalidStringSize(len));
        }

        if len == 0 {
            return Ok(None);
        }

        if len < 0 {
            let len = -len;

            let len = len * size_of::<u16>() as i32 - 2;
            let mut buf = vec![0u8; len as usize];
            self.read_exact(&mut buf)?;

            let terminator = self.read_u16::<LittleEndian>()?;
            if terminator != 0 {
                return Err(FStringError::InvalidStringTerminator);
            }

            String::from_utf16(
                &buf.chunks(2)
                    .map(|e| u16::from_le_bytes([e[0], e[1]]))
                    .collect::<Vec<_>>(),
            )
            .map(Some)
            .map_err(|e| e.into())
        } else {
            let mut buf = vec![0u8; len as usize - 1];
            self.read_exact(&mut buf)?;

            let terminator = self.read_u8()?;
            if terminator != 0 {
                return Err(FStringError::InvalidStringTerminator);
            }

            String::from_utf8(buf).map(Some).map_err(|e| e.into())
        }
    }

    fn read_bool(&mut self) -> io::Result<bool> {
        let res = self.read_u8()?;
        Ok(res > 0)
    }
}