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

pub mod compression;
mod entry;
pub mod error;
mod header;
mod index;
pub mod pakmemory;
pub mod pakreader;
pub mod pakversion;
pub mod pakwriter;

pub use pakmemory::PakMemory;
pub use pakreader::PakReader;
pub use pakwriter::PakWriter;

pub use compression::Compression;
pub use error::PakError;

pub(crate) const PAK_MAGIC: u32 = u32::from_be_bytes([0xE1, 0x12, 0x6F, 0x5A]);

pub(crate) fn hash(data: &[u8]) -> [u8; 20] {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}
