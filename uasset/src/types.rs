#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Vector<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Vector { x, y, z }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T> Vector4<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Vector4 { x, y, z, w }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Color<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T> Color<T> {
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Color { r, g, b, a }
    }
}

impl Color<u8> {
    pub fn from_argb(argb: i32) -> Self {
        Color::new(
            ((argb >> 24) & 0xff) as u8,
            ((argb >> 16) & 0xff) as u8,
            ((argb >> 8) & 0xff) as u8,
            (argb & 0xff) as u8,
        )
    }

    pub fn to_argb(&self) -> i32 {
        ((self.r as i32) << 24) | ((self.g as i32) << 16) | ((self.b as i32) << 8) | self.a as i32
    }
}

pub struct Transform<T> {
    pub rotation: Vector4<T>,
    pub translation: Vector<T>,
    pub scale: Vector<T>,
}

impl<T> Transform<T> {
    pub fn new(rotation: Vector4<T>, translation: Vector<T>, scale: Vector<T>) -> Self {
        Transform {
            rotation,
            translation,
            scale,
        }
    }
}
