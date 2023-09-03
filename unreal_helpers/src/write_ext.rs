//! Extension for anything that implements `Write` to more easily write Unreal data formats.

use std::error::Error;
use std::io::{self, Write};
use std::mem::size_of;

use byteorder::{WriteBytesExt, LE};

use crate::error::FStringError;

/// Extension for anything that implements `Write` to more easily write Unreal data formats.
pub trait UnrealWriteExt {
    /// Write bool as u8.
    fn write_bool(&mut self, value: bool) -> io::Result<()>;

    /// Write an array of type `T` by running the provided function for
    /// each element of the provided array which serializes `T` into the writer.
    /// The length of the array will be written at the start as an `i32`.
    fn write_array<T, E: Error + From<io::Error>>(
        &mut self,
        array: &[T],
        f: impl FnMut(&mut Self, &T) -> Result<(), E>,
    ) -> Result<(), E>;

    /// Write a guid.
    #[cfg(feature = "guid")]
    fn write_guid(&mut self, guid: &crate::Guid) -> io::Result<()>;

    /// Write string of format \<length i32\>\<string\>\<null\>.
    fn write_fstring(&mut self, string: Option<&str>) -> Result<usize, FStringError>;
}

impl<W: Write> UnrealWriteExt for W {
    fn write_bool(&mut self, value: bool) -> io::Result<()> {
        self.write_u8(match value {
            true => 1,
            false => 0,
        })
    }

    fn write_array<T, E: Error + From<io::Error>>(
        &mut self,
        array: &[T],
        mut f: impl FnMut(&mut Self, &T) -> Result<(), E>,
    ) -> Result<(), E> {
        self.write_i32::<LE>(array.len() as i32)?;
        for value in array {
            f(self, value)?;
        }
        Ok(())
    }

    #[cfg(feature = "guid")]
    fn write_guid(&mut self, guid: &crate::Guid) -> io::Result<()> {
        self.write_all(&guid.0)
    }

    fn write_fstring(&mut self, string: Option<&str>) -> Result<usize, FStringError> {
        if let Some(string) = string {
            let is_unicode = string.len() != string.chars().count();

            if is_unicode {
                let utf16 = string.encode_utf16().collect::<Vec<_>>();

                // this is safe because we know that string is utf16 and therefore can easily be aligned to u8
                // this is also faster than alternatives without unsafe block
                let (_, aligned, _) = unsafe { utf16.align_to::<u8>() };

                self.write_i32::<LE>(-(aligned.len() as i32 / 2) - 1)?;
                self.write_all(aligned)?;

                self.write_all(&[0u8; 2])?;
                Ok(size_of::<i32>() + aligned.len())
            } else {
                self.write_i32::<LE>(string.len() as i32 + 1)?;
                let bytes = string.as_bytes();
                self.write_all(bytes)?;
                self.write_all(&[0u8; 1])?;

                Ok(size_of::<i32>() + bytes.len() + 1)
            }
        } else {
            self.write_i32::<LE>(0)?;
            Ok(size_of::<i32>())
        }
    }
}
