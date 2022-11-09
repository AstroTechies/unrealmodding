#![deny(missing_docs)]
/*
Unreal pak format (version 8)
File parts:
    - entries
        - entry header
        - entry data
    - index
        - entry index
            - entry name
            - entry header
    - footer
*/

//! # unreal_pak
//!
//! Utility crate for working with Unreal Engine .pak files.
//! Supports both reading and writing and aims to support all pak versions.
//! Encrytion is currently unsupported

mod buf_ext;
mod entry;
pub mod error;
mod header;
mod index;
pub mod pakfile;
//pub mod pakmemory;
pub mod pakversion;

pub use pakfile::PakFile;
//pub use pakmemory::PakMemory;

pub(crate) const PAK_MAGIC: u32 = u32::from_be_bytes([0xe1, 0x12, 0x6f, 0x5a]);

/// Enum representing which compression method is being used for an entry
#[derive(
    PartialEq, Eq, Debug, Clone, Copy, num_enum::IntoPrimitive, num_enum::TryFromPrimitive,
)]
#[repr(i32)]
pub enum CompressionMethod {
    /// No comprssion
    None = 0,
    /// Standard Zlib comprssion
    Zlib = 1,
    /// BiasMemory comprssion
    BiasMemory = 2,
    /// BiasSpeed comprssion
    BiasSpeed = 3,
    /// Unknown comprssion
    Unknown = 255,
}
