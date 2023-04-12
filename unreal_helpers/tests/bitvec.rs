#![cfg(feature = "bitvec")]

use bitvec::{bitvec, order::Lsb0};
use unreal_helpers::BitVecExt;

#[test]
fn test_set_range_from_range() {
    let mut vec = bitvec![1, 1, 1, 0, 0, 1, 1];
    let range = bitvec![1, 1, 0, 0];

    vec.set_range_from_range(3, 4, &range, 0);

    assert_eq!(vec, bitvec![1, 1, 1, 1, 1, 0, 0]);
}
