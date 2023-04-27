//! Package object index
//! Usually used inside Zen

/// Package object index
/// Same as PackageIndex but for Zen
pub struct PackageObjectIndex {}

impl PackageObjectIndex {
    /// Index bit count
    pub const INDEX_BITS: u64 = 62;
    /// Index mask
    pub const INDEX_MASK: u64 = (1u64 << PackageObjectIndex::INDEX_BITS) - 1u64;
}
