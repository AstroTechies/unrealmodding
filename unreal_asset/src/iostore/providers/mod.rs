//! IoStore .ucas file providers
//!
//! These are used to allow different .ucas file reading strategies
//! e.g. Memory mapping from disk, full read into memory, etc.

use std::io::{Read, Seek};

use crate::error::Error;

pub mod file;
pub mod memory;

/// IoStore .ucas file provider trait
pub trait IoStoreProvider<R: Read + Seek> {
    /// Create a reader for a container with the given file name
    fn create_reader_for_file(&self, file_name: &str) -> Result<R, Error>;
}
