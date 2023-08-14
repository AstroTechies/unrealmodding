//! Value alignment helpers

/// Align a value with an alignment
pub fn align(val: u64, alignment: u64) -> u64 {
    (val + alignment - 1) & !alignment.overflowing_sub(1).0
}
