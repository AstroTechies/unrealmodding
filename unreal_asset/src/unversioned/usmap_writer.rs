use std::io;

pub trait UsmapWriter {
    fn write_i8(&mut self, value: i8) -> Result<(), io::Error>;
    fn write_u8(&mut self, value: u8) -> Result<(), io::Error>;
    fn write_i16(&mut self, value: i16) -> Result<(), io::Error>;
    fn write_u16(&mut self, value: u16) -> Result<(), io::Error>;
    fn write_i32(&mut self, value: i32) -> Result<(), io::Error>;
    fn write_u32(&mut self, value: u32) -> Result<(), io::Error>;
    fn write_i64(&mut self, value: i64) -> Result<(), io::Error>;
    fn write_u64(&mut self, value: u64) -> Result<(), io::Error>;
    fn write_f32(&mut self, value: f32) -> Result<(), io::Error>;
    fn write_f64(&mut self, value: f64) -> Result<(), io::Error>;
    fn write_string(&mut self, value: &str) -> Result<usize, io::Error>;
    fn write_name(&mut self, name: &str) -> Result<(), io::Error>;
}
