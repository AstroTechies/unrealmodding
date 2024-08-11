//! Chain for chaining two `Read` + `Seek` implementations

use std::io::{Read, Result, Seek, SeekFrom};

/// Chain for chaining two `Read` + `Seek` implementations
pub struct Chain<C: Read + Seek> {
    first: C,
    second: Option<C>,
    first_len: u64,
    second_len: u64,
    pos: u64,
}

impl<C: Read + Seek> Chain<C> {
    /// Create a new chain
    pub fn new(mut first: C, mut second: Option<C>) -> Self {
        // ignore errors for now
        let first_len = first.seek(SeekFrom::End(0)).unwrap_or_default();
        first.rewind().unwrap_or_default();
        let second_len = match second.as_mut() {
            Some(sec) => {
                let len = sec.seek(SeekFrom::End(0)).unwrap_or_default();
                sec.rewind().unwrap_or_default();
                len
            }
            None => 0,
        };
        Self {
            first,
            second,
            first_len,
            second_len,
            pos: 0,
        }
    }
}

impl<C: Read + Seek> Read for Chain<C> {
    // this is an implementation of read so clippy complaining about use of read is stupid
    #[allow(clippy::unused_io_amount)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.second.as_mut() {
            Some(sec) => {
                let len_read = match self.pos >= self.first_len {
                    true => sec.read(buf)?,
                    false => {
                        let len = buf.len() as u64;
                        let to_end = self.first_len - self.pos;
                        match to_end >= len {
                            true => self.first.read(buf)?,
                            false => {
                                let mut first = vec![0; to_end as usize];
                                let excess = len - to_end;
                                let mut second = vec![
                                    0;
                                    match excess > self.second_len {
                                        true => self.second_len,
                                        false => excess,
                                    } as usize
                                ];
                                self.first.read_exact(&mut first)?;
                                sec.read_exact(&mut second)?;
                                first.append(&mut second);
                                first.as_slice().read(buf)?
                            }
                        }
                    }
                };
                self.pos += len_read as u64;
                Ok(len_read)
            }
            None => self.first.read(buf),
        }
    }
}

impl<C: Read + Seek> Seek for Chain<C> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match self.second.as_mut() {
            Some(sec) => match pos {
                SeekFrom::Start(offset) => {
                    self.pos = match offset < self.first_len {
                        true => self.first.seek(pos)?,
                        false => {
                            self.first_len + sec.seek(SeekFrom::Start(offset - self.first_len))?
                        }
                    };
                    Ok(offset)
                }
                SeekFrom::End(offset) => self.seek(SeekFrom::Start(
                    ((self.first_len + self.second_len) as i64 + offset) as u64,
                )),
                SeekFrom::Current(offset) => {
                    self.seek(SeekFrom::Start((self.pos as i64 + offset) as u64))
                }
            },
            None => self.first.seek(pos),
        }
    }
}
