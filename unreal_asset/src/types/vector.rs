//! Vector/Quat/etc. types
//!

/// Vector
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Vector<T> {
    /// X component
    pub x: T,
    /// Y component
    pub y: T,
    /// Z component
    pub z: T,
}

impl<T> Vector<T> {
    /// Create a new `Vector` instance
    pub fn new(x: T, y: T, z: T) -> Self {
        Vector { x, y, z }
    }
}

/// Vector4
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Vector4<T> {
    /// X component
    pub x: T,
    /// Y component
    pub y: T,
    /// Z component
    pub z: T,
    /// Real component
    pub w: T,
}

impl<T> Vector4<T> {
    /// Create a new `Vector4` instance
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Vector4 { x, y, z, w }
    }
}

/// RGBA Color
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Color<T> {
    /// Red
    pub r: T,
    /// Green
    pub g: T,
    /// Blue
    pub b: T,
    /// Alpha
    pub a: T,
}

impl<T> Color<T> {
    /// Create a new `Color` instance
    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Color { r, g, b, a }
    }
}

impl Color<u8> {
    /// Create a new `Color<u8>` instance from an argb int
    pub fn from_argb(argb: i32) -> Self {
        Color::new(
            ((argb >> 24) & 0xff) as u8,
            ((argb >> 16) & 0xff) as u8,
            ((argb >> 8) & 0xff) as u8,
            (argb & 0xff) as u8,
        )
    }

    /// Convert to argb int
    pub fn to_argb(&self) -> i32 {
        ((self.r as i32) << 24) | ((self.g as i32) << 16) | ((self.b as i32) << 8) | self.a as i32
    }
}

/// Transform
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Transform<T> {
    /// Rotation
    pub rotation: Vector4<T>,
    /// Translation
    pub translation: Vector<T>,
    /// Scale
    pub scale: Vector<T>,
}

impl<T> Transform<T> {
    /// Create a new `Transform` instance
    pub fn new(rotation: Vector4<T>, translation: Vector<T>, scale: Vector<T>) -> Self {
        Transform {
            rotation,
            translation,
            scale,
        }
    }
}
