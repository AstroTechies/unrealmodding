use std::io;

pub struct Chain<C: io::Read + io::Seek> {
    first: C,
    second: Option<C>,
    first_len: u64,
    second_len: u64,
    pos: u64,
}

impl<C: io::Read + io::Seek> Chain<C> {
    pub fn new(mut first: C, mut second: Option<C>) -> Self {
        // ignore errors for now
        let first_len = first.seek(io::SeekFrom::End(0)).unwrap_or_default();
        first.rewind().unwrap_or_default();
        let second_len = match second.as_mut() {
            Some(sec) => {
                let len = sec.seek(io::SeekFrom::End(0)).unwrap_or_default();
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

impl<C: io::Read + io::Seek> io::Read for Chain<C> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.second.as_mut() {
            Some(sec) => {
                let len_read = match self.pos >= self.first_len {
                    true => sec.read(buf)?,
                    false => {
                        let len = buf.len();
                        let to_end = (self.first_len - self.pos) as usize;
                        match to_end >= len {
                            true => self.first.read(buf)?,
                            false => {
                                let mut first = vec![0; to_end];
                                let mut second = vec![0; len - to_end];
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

impl<C: io::Read + io::Seek> io::Seek for Chain<C> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match self.second.as_mut() {
            Some(sec) => match pos {
                io::SeekFrom::Start(offset) => {
                    self.pos = match offset < self.first_len {
                        true => self.first.seek(pos)?,
                        false => {
                            self.first_len
                                + sec.seek(io::SeekFrom::Start(offset - self.first_len))?
                        }
                    };
                    Ok(offset)
                }
                io::SeekFrom::End(offset) => self.seek(io::SeekFrom::Start(
                    ((self.first_len + self.second_len) as i64 + offset) as u64,
                )),
                io::SeekFrom::Current(offset) => {
                    self.seek(io::SeekFrom::Start((self.pos as i64 + offset) as u64))
                }
            },
            None => self.first.seek(pos),
        }
    }
}

#[test]
fn read() {
    use io::Read;
    let mut v = Vec::with_capacity(12);
    Chain::new(
        io::Cursor::new(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        Some(io::Cursor::new(vec![0, 1, 2, 3])),
    )
    .read_to_end(&mut v)
    .unwrap();
    assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3]);
}

#[test]
fn seek() {
    let mut chain = Chain::new(
        io::Cursor::new(vec![0, 1, 2, 3]),
        Some(io::Cursor::new(vec![4, 5, 6, 7])),
    );
    let mut read_at = |pos| {
        use byteorder::ReadBytesExt;
        use io::Seek;
        chain.seek(pos)?;
        chain.read_u8()
    };
    assert_eq!(read_at(io::SeekFrom::Start(0)).unwrap(), 0);
    assert!(read_at(io::SeekFrom::Start(8)).is_err());
    assert_eq!(read_at(io::SeekFrom::Current(-1)).unwrap(), 7);
    assert_eq!(read_at(io::SeekFrom::Current(-5)).unwrap(), 3);
    assert_eq!(read_at(io::SeekFrom::End(-4)).unwrap(), 4);
    assert!(read_at(io::SeekFrom::End(-12)).is_err());
}
