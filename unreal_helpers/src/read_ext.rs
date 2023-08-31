//! Extension for anything that implements `Read` to more easily read Unreal data formats.

use std::io::{self, Read, Seek};
use std::mem::size_of;

use byteorder::{ReadBytesExt, LE};

use crate::error::FStringError;

/// Extension for anything that implements `Read` to more easily read Unreal data formats.
pub trait UnrealReadExt {
    /// Read u8 as bool.
    fn read_bool(&mut self) -> io::Result<bool>;

    /// Read a `Vec` of length `len` from the reader.
    fn read_vec(&mut self, len: usize) -> io::Result<Vec<u8>>;

    /// Read an array of type `T` by consuming bytes of the reader `n` times and
    /// parsing them into `T` using the provided function where the value of `n`
    /// is determined by first `i32` aread from the reader.
    fn read_array<T>(&mut self, f: impl FnMut(&mut Self) -> io::Result<T>) -> io::Result<Vec<T>>;

    /// Read a guid.
    #[cfg(feature = "guid")]
    fn read_guid(&mut self) -> io::Result<crate::Guid>;

    /// Read string of format \<length i32\>\<string\>\<null\>.
    fn read_fstring(&mut self) -> Result<Option<String>, FStringError>;
}

impl<R: Read + Seek> UnrealReadExt for R {
    fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    fn read_vec(&mut self, len: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; len];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_array<T>(
        &mut self,
        mut f: impl FnMut(&mut Self) -> io::Result<T>,
    ) -> io::Result<Vec<T>> {
        let mut buf = Vec::with_capacity(self.read_u32::<LE>()? as usize);
        for _ in 0..buf.capacity() {
            buf.push(f(self)?);
        }
        Ok(buf)
    }

    fn read_fstring(&mut self) -> Result<Option<String>, FStringError> {
        let len = self.read_i32::<LE>()?;

        let (len, is_wide) = match len < 0 {
            true => (-len, true),
            false => (len, false),
        };
        read_fstring_len(self, len, is_wide)
    }

    #[cfg(feature = "guid")]
    fn read_guid(&mut self) -> io::Result<crate::Guid> {
        let mut buf = [0u8; 16];
        self.read_exact(&mut buf)?;
        Ok(crate::Guid(buf))
    }
}

/// Read string of format \<string\>\<null\> when length and encoding is already known.
#[inline(always)]
pub fn read_fstring_len<R: Read + Seek>(
    reader: &mut R,
    len: i32,
    is_wide: bool,
) -> Result<Option<String>, FStringError> {
    if len == 0 {
        return Ok(None);
    }

    let result = read_fstring_len_noterm(reader, len.saturating_sub(1), is_wide)?;

    if is_wide {
        let terminator = reader.read_u16::<LE>()?;
        if terminator != 0 {
            return Err(FStringError::InvalidStringTerminator(
                terminator,
                reader.stream_position()?,
            ));
        }
    } else {
        let terminator = reader.read_u8()?;
        if terminator != 0 {
            return Err(FStringError::InvalidStringTerminator(
                terminator as u16,
                reader.stream_position()?,
            ));
        }
    }

    Ok(result)
}

/// Read string of format \<string\> when length and encoding is already known.
#[inline(always)]
pub fn read_fstring_len_noterm<R: Read + Seek>(
    reader: &mut R,
    len: i32,
    is_wide: bool,
) -> Result<Option<String>, FStringError> {
    if !(-131072..=131072).contains(&len) {
        return Err(FStringError::InvalidStringSize(
            len,
            reader.stream_position()?,
        ));
    }

    if is_wide {
        let len = len * size_of::<u16>() as i32;
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;

        String::from_utf16(
            &buf.chunks(2)
                .map(|e| u16::from_le_bytes([e[0], e[1]]))
                .collect::<Vec<_>>(),
        )
        .map(Some)
        .map_err(|e| e.into())
    } else {
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;

        String::from_utf8(buf).map(Some).map_err(|e| e.into())
    }
}
