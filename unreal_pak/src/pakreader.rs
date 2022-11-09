//! PakFile data structure for reading large pak files

use std::collections::BTreeMap;
use std::io::{BufReader, Read, Seek};

use crate::entry::read_entry;
use crate::error::PakError;
use crate::header::Header;
use crate::index::Index;
use crate::pakversion::PakVersion;

/// An Unreal pak file reader with it's data kept on disk and only read on demand.
#[derive(Debug)]
pub struct PakReader<'data, R>
where
    &'data R: Read + Seek,
{
    /// version of the pak file format this one is using
    pak_version: PakVersion,
    /// mount point (Unreal stuff)
    pub mount_point: String,
    entries: BTreeMap<String, Header>,
    reader: BufReader<&'data R>,
}

impl<'data, R> PakReader<'data, R>
where
    &'data R: Read + Seek,
{
    /// Creates a new `PakFile` configured to read files.
    pub fn new(reader: &'data R) -> Self {
        Self {
            pak_version: PakVersion::PakFileVersionInvalid,
            mount_point: "".to_owned(),
            entries: BTreeMap::new(),
            reader: BufReader::new(reader),
        }
    }

    /// Load the entry info contained in the footer into memory to start reading individual entries.
    pub fn load_index(&mut self) -> Result<(), PakError> {
        let index = Index::read(&mut self.reader)?;

        self.pak_version = index.footer.pak_version;
        self.mount_point = index.mount_point.clone();
        //? maybe also store compression somehow?

        for (name, header) in index.entries {
            self.entries.insert(name, header);
        }

        Ok(())
    }

    /// Returns the names of all entries which have been found.
    pub fn get_entry_names(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    /// Reads an entry from the pak on disk into memory and returns it's data.
    pub fn read_entry(&mut self, name: &String) -> Result<Vec<u8>, PakError> {
        let header = self
            .entries
            .get(name)
            .ok_or_else(|| PakError::entry_not_found(name.clone()))?;
        read_entry(&mut self.reader, self.pak_version, header.offset)
    }
}
