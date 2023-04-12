#![deny(missing_docs)]

//! Various small functions to make working with Unreal data formats easier.

pub mod error;

#[cfg(feature = "bitvec")]
pub mod bitvec_ext;
#[cfg(feature = "bitvec")]
pub use bitvec_ext::BitVecExt;

#[cfg(feature = "read_write")]
pub mod read_ext;
#[cfg(feature = "read_write")]
pub use read_ext::UnrealReadExt;

#[cfg(feature = "read_write")]
pub mod write_ext;
#[cfg(feature = "read_write")]
pub use write_ext::UnrealWriteExt;

#[cfg(feature = "path")]
pub mod path;
#[cfg(feature = "path")]
pub use path::game_to_absolute;
