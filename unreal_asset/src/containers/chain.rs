use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

pub struct Chain<C: Read + Seek> {
    middle: u64,
    size: u64,
    first: C,
    second: Option<C>,
    pos: u64,
}

impl<C: Read + Seek> Chain<C> {
    pub fn new(mut first: C, mut second: Option<C>) -> Self {
        let middle = first.seek(SeekFrom::End(0)).unwrap_or_default();
        let size = match second.as_mut() {
            Some(second) => second.seek(SeekFrom::End(0)).unwrap_or_default(),
            None => middle,
        };
        Self {
            middle,
            size,
            first,
            second,
            pos: 0,
        }
    }
}

impl<C: Read + Seek> Read for Chain<C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.pos = if self.pos > self.middle {
            match self.second.as_mut() {
                Some(second) => second.read(buf)? as u64 + self.middle,
                None => return Err(Error::new(ErrorKind::UnexpectedEof, "read past eof")),
            }
        } else if buf.len() as u64 <= self.middle - self.pos {
            self.first.read(buf)? as u64
        } else {
            let mut part1 = Vec::new();
            let mut seek = self.first.read_to_end(&mut part1).unwrap_or_default() as u64;
            let mut part2 = Vec::with_capacity(buf.len() - seek as usize);
            seek += part2.capacity() as u64;
            match self.second.as_mut() {
                Some(second) => second.read_exact(&mut part2).unwrap_or_default(),
                None => return Err(Error::new(ErrorKind::UnexpectedEof, "read past eof")),
            }
            part1.append(&mut part2);
            buf.copy_from_slice(&part1);
            self.pos + seek
        };
        Ok(self.pos as usize)
    }
}

impl<C: Read + Seek> Seek for Chain<C> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.pos = match pos {
            SeekFrom::Start(offset) => match offset > self.size {
                true => return Err(Error::new(ErrorKind::UnexpectedEof, "seeked out of file")),
                false => offset,
            },
            SeekFrom::End(offset) => match offset > self.size as i64 {
                true => return Err(Error::new(ErrorKind::UnexpectedEof, "seeked out of file")),
                false => self.size - offset as u64,
            },
            SeekFrom::Current(offset) => match self.pos as i64 + offset {
                res if res > self.size as i64 || res.is_negative() => {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "seeked out of file"))
                }
                res => res as u64,
            },
        };
        Ok(self.pos)
    }
}
