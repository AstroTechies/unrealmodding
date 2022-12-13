//! Vector/Quat/etc. types
//!
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

impl<T: Copy> From<[T; 3]> for Vector<T> {
    fn from(src: [T; 3]) -> Self {
        Self {
            x: src[0],
            y: src[1],
            z: src[2],
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

impl<T: Copy> From<[T; 4]> for Vector4<T> {
    fn from(src: [T; 4]) -> Self {
        Self {
            x: src[0],
            y: src[1],
            z: src[2],
            w: src[3],
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Color<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T: Copy> From<[T; 4]> for Color<T> {
    fn from(src: [T; 4]) -> Self {
        Self {
            r: src[0],
            g: src[1],
            b: src[2],
            a: src[3],
        }
    }
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
