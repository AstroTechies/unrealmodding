use std::io;

pub trait UsmapWriter {
    fn write_i8(&mut self, value: i8) -> io::Result<()>;
    fn write_u8(&mut self, value: u8) -> io::Result<()>;
    fn write_i16(&mut self, value: i16) -> io::Result<()>;
    fn write_u16(&mut self, value: u16) -> io::Result<()>;
    fn write_i32(&mut self, value: i32) -> io::Result<()>;
    fn write_u32(&mut self, value: u32) -> io::Result<()>;
    fn write_i64(&mut self, value: i64) -> io::Result<()>;
    fn write_u64(&mut self, value: u64) -> io::Result<()>;
    fn write_f32(&mut self, value: f32) -> io::Result<()>;
    fn write_f64(&mut self, value: f64) -> io::Result<()>;
    fn write_fstring(&mut self, value: &str) -> io::Result<usize>;
    fn write_name(&mut self, name: &str) -> io::Result<()>;
}
