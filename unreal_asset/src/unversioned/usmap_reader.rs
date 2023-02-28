use std::io;

pub trait UsmapReader {
    fn read_i8(&mut self) -> io::Result<i8>;
    fn read_u8(&mut self) -> io::Result<u8>;
    fn read_i16(&mut self) -> io::Result<i16>;
    fn read_u16(&mut self) -> io::Result<u16>;
    fn read_i32(&mut self) -> io::Result<i32>;
    fn read_u32(&mut self) -> io::Result<u32>;
    fn read_i64(&mut self) -> io::Result<i64>;
    fn read_u64(&mut self) -> io::Result<u64>;
    fn read_f32(&mut self) -> io::Result<f32>;
    fn read_f64(&mut self) -> io::Result<f64>;
    fn read_fstring(&mut self) -> io::Result<Option<String>>;
    fn read_name(&mut self) -> io::Result<String>;
}
