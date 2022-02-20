
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