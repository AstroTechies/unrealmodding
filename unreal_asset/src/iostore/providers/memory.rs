//! IoStore .ucas file provider using in-memory data

use std::{collections::HashMap, io::Cursor};

use crate::error::{Error, IoStoreError};

use super::IoStoreProvider;

/// File provider using in-memory data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoStoreMemoryProvider<'data> {
    mappings: HashMap<String, &'data [u8]>,
}

impl<'data> IoStoreMemoryProvider<'data> {
    /// Creae a new `IoStoreMemoryProvider` instance
    pub fn new(mappings: HashMap<String, &'data [u8]>) -> Self {
        IoStoreMemoryProvider { mappings }
    }

    /// Add a new file mapping to this instance
    pub fn add_mapping(&mut self, name: &str, data: &'data [u8]) {
        self.mappings.insert(name.to_owned(), data);
    }

    /// Remove a file mapping from this instance
    pub fn remove_mapping(&mut self, name: &str) {
        self.mappings.remove(name);
    }
}

impl<'data> IoStoreProvider<Cursor<&'data [u8]>> for IoStoreMemoryProvider<'data> {
    fn create_reader_for_file(&self, file_name: &str) -> Result<Cursor<&'data [u8]>, Error> {
        if let Some(mapping) = self.mappings.get(file_name) {
            return Ok(Cursor::new(mapping));
        }

        Err(IoStoreError::NoFile(file_name.to_string().into_boxed_str()).into())
    }
}
