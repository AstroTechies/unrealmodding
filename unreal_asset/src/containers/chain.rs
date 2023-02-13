use std::io;

pub struct Chain<C: io::Read + io::Seek> {
    first: C,
    second: Option<C>,
    pos: u64,
}

impl<C: io::Read + io::Seek> Chain<C> {
    pub fn new(first: C, second: Option<C>) -> Self {
        Self {
            first,
            second,
            pos: 0,
        }
    }
}

impl<C: io::Read + io::Seek> io::Read for Chain<C> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.second {
            Some(_) => unimplemented!(),
            None => self.first.read(buf),
        }
    }
}

impl<C: io::Read + io::Seek> io::Seek for Chain<C> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match self.second {
            Some(_) => unimplemented!(),
            None => self.first.seek(pos),
        }
    }
}
