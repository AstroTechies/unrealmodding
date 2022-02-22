
#[derive(Debug)]
pub struct Vector<T> {
    x: T,
    y: T,
    z: T
}

impl<T> Vector<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Vector{ x, y, z }
    }
}

#[derive(Debug)]
pub struct Color<T> {
    r: T,
    g: T,
    b: T,
    a: T
}

impl<T> Color<T> {
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Color { r, g, b, a }
    }
}

impl Color<u8> {
    pub fn from_argb(argb: i32) -> Self {
        Color::new((argb >> 24) & 0xff, (argb >> 16) & 0xff, (argb >> 8) & 0xff, argb & 0xff)
    }
}