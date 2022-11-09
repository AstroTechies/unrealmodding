//! PakMemory data structure for more flexible pak files

use std::collections::BTreeMap;
use std::io::{Read, Seek, Write};

use crate::entry::{read_entry, write_entry};
use crate::error::PakError;
use crate::index::{Footer, Index};
use crate::pakversion::PakVersion;
use crate::CompressionMethod;

/// A Unreal Pak file which keeps all of it's data in memory.
/// It allows reading and writing of the same entries before comitting the file to disk.
#[derive(Debug)]
pub struct PakMemory {
    /// version of the pak file format this one is using
    pub pak_version: PakVersion,
    /// mount point (Unreal stuff)
    pub mount_point: String,
    /// the compression method preferred for this file
    pub compression: CompressionMethod,
    /// the compression block size
    pub block_size: u32,
    entries: BTreeMap<String, Vec<u8>>,
}

impl PakMemory {
    /// Creates a new `PakMemory`.
    pub fn new(pak_version: PakVersion) -> Self {
        Self {
            pak_version,
            mount_point: "../../../".to_owned(),
            compression: CompressionMethod::default(),
            block_size: 0x010000,
            entries: BTreeMap::new()
        }
    }

    /// Loads the data contained in the pak file in the reader into this PakMemory
    pub fn load<R: Read + Seek>(&mut self, mut reader: &mut R) -> Result<(), PakError> {
        let index = Index::read(reader)?;

        self.pak_version = index.footer.pak_version;
        self.mount_point = index.mount_point.clone();

        for (name, header) in index.entries {
            self.entries.insert(name, read_entry(&mut reader, self.pak_version, header.offset)?);
        }

        Ok(())
    }

    /// Create a new PakMemory based on the data of the reader.
    pub fn load_from<R: Read + Seek>(reader: &mut R) -> Result<Self, PakError> {
        let mut pak_memory = Self::new(PakVersion::PakFileVersionInvalid);
        pak_memory.load(reader)?;
        Ok(pak_memory)
    }

    /// Returns the names of all entries stored in this PakMemory.
    pub fn get_entry_names(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    /// Get the data of an entry.
    pub fn get_entry(&self, name: &String) -> Option<&Vec<u8>> {
        self.entries.get(name)
    }

    /// Set the data for an entry
    pub fn set_entry(&mut self, name: String, data: Vec<u8>) {
        self.entries.insert(name, data);
    }

    /// Finish writing the pak file by writing index and footer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<(), PakError> {
        let mut written_entries = Vec::new();

        for (name, data) in self.entries.iter() {
            let header = write_entry(
                writer,
                self.pak_version,
                data,
                self.compression,
                self.block_size,
            )?;
            written_entries.push((name.clone(), header));
        }

        let footer = Footer {
            pak_version: self.pak_version,
            // these are set in write_index
            index_offset: 0,
            index_size: 0,
            index_hash: [0u8; 20],
            index_encrypted: Some(false),
            encryption_key_guid: Some([0u8; 0x10]),
        };

        let index = Index {
            mount_point: self.mount_point.clone(),
            entries: written_entries,
            footer,
        };

        Index::write(writer, index)
    }
}
