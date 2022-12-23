#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameNumber {
    pub value: i32,
}

impl FrameNumber {
    pub fn new(value: i32) -> Self {
        FrameNumber { value }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FrameRate {
    pub numerator: i32,
    pub denominator: i32,
}

impl FrameRate {
    pub fn new(numerator: i32, denominator: i32) -> Self {
        FrameRate {
            numerator,
            denominator,
        }
    }
}
