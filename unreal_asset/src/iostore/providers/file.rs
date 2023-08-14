//! IoStore .ucas file provider using a folder

use std::{fs::File, path::PathBuf};

use crate::error::Error;

use super::IoStoreProvider;

/// File provider from a folder
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreFileProvider {
    folder: PathBuf,
}

impl IoStoreFileProvider {
    /// Create a new `IoStoreFileProvider` instance
    pub fn new(folder: PathBuf) -> Self {
        IoStoreFileProvider { folder }
    }
}

impl IoStoreProvider<File> for IoStoreFileProvider {
    fn create_reader_for_file(&self, file_name: &str) -> Result<File, Error> {
        Ok(File::open(self.folder.join(file_name))?)
    }
}
