#![deny(missing_docs)]

//! Various small functions to make working with Unreal data formats easier.
//!
//! ## Feature flags
//!
//! All content in this crate is hidden behind feature flags.
//!
//! - `read_write`: Enables extension Traits [`UnrealReadExt`] and [`UnrealWriteExt`]
//!                 which help with parsing Unreal data formats.
//! - `path`: Enables [`game_to_absolute`] function.
//! - `guid`: Enables [`Guid`] type.
//! - `serde`: Enables `serde` support for [`Guid`] type.
//! - `bitvec`: Enables extension Trait [`BitVecExt`].

#[cfg(feature = "bitvec")]
pub mod bitvec_ext;
#[cfg(feature = "bitvec")]
pub use bitvec_ext::BitVecExt;

pub mod error;

#[cfg(feature = "guid")]
pub mod guid;
#[cfg(feature = "guid")]
pub use guid::Guid;

#[cfg(feature = "path")]
pub mod path;
#[cfg(feature = "path")]
pub use path::game_to_absolute;

#[cfg(feature = "read_write")]
pub mod read_ext;
#[cfg(feature = "read_write")]
pub use read_ext::UnrealReadExt;

#[cfg(feature = "read_write")]
pub mod write_ext;
#[cfg(feature = "read_write")]
pub use write_ext::UnrealWriteExt;
