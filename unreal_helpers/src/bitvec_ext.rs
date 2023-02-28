//! BitVec extensions for easier deserialization

use bitvec::{order::BitOrder, prelude::BitVec, store::BitStore};

/// BitVec extention for setting ranges.
pub trait BitVecExt<T: BitStore, O: BitOrder> {
    /// BitVec extention for setting a range from another range.
    fn set_range_from_range(&mut self, index: i32, num: i32, range: &BitVec<T, O>, offset: i32);
}

impl<T: BitStore, O: BitOrder> BitVecExt<T, O> for BitVec<T, O> {
    fn set_range_from_range(
        &mut self,
        index: i32,
        num: i32,
        range: &BitVec<T, O>,
        read_offset: i32,
    ) {
        for i in 0..num {
            self.set(
                (index + i) as usize,
                range
                    .get((read_offset + i) as usize)
                    .as_deref()
                    .copied()
                    .unwrap_or(false),
            )
        }
    }
}
