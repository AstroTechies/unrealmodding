//! PakFile data structure for writing large pak files

use std::collections::BTreeMap;
use std::io::{BufWriter, Seek, Write};

use crate::entry::write_entry;
use crate::error::PakError;
use crate::header::Header;
use crate::index::{Footer, Index};
use crate::pakversion::PakVersion;
use crate::CompressionMethod;

/// An Unreal pak file writer which allows incrementally writing data.
/// Good for working with very large files, but it has restrictions when it
/// comes to writing files. For a more flexible alternative see \<insert doc link\> PakMemory
#[derive(Debug)]
pub struct PakWriter<'data, W>
where
    &'data W: Write + Seek,
{
    /// version of the pak file format this one is using
    pub pak_version: PakVersion,
    /// mount point (Unreal stuff)
    pub mount_point: String,
    /// the compression method preferred for this file
    pub compression: CompressionMethod,
    /// the compression block size
    pub block_size: u32,
    entries: BTreeMap<String, Header>,
    writer: BufWriter<&'data W>,
}

impl<'data, W> PakWriter<'data, W>
where
    &'data W: Write + Seek,
{
    /// Creates a new `PakFile` configured to write files.
    pub fn new(writer: &'data W, pak_version: PakVersion, compression: CompressionMethod) -> Self {
        Self {
            pak_version,
            mount_point: "../../../".to_owned(),
            compression,
            block_size: 0x010000,
            entries: BTreeMap::new(),
            writer: BufWriter::new(writer),
        }
    }

    /// Returns the names of all entries which have been found.
    pub fn get_entry_names(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    /// Writes the given data into the pak file on disk.
    /// Writes should happen in an aplphabetical order.
    pub fn write_entry(&mut self, name: &String, data: &Vec<u8>) -> Result<(), PakError> {
        if self.entries.contains_key(name) {
            return Err(PakError::double_write(name.clone()));
        }

        let header = write_entry(
            &mut self.writer,
            self.pak_version,
            data,
            self.compression,
            self.block_size,
        )?;
        self.entries.insert(name.clone(), header);

        Ok(())
    }

    /// Finish writing the pak file by writing index and footer
    pub fn finish_write(mut self) -> Result<(), PakError> {
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
            mount_point: self.mount_point,
            entries: self.entries.into_iter().collect::<Vec<_>>(),
            footer,
        };

        Index::write(&mut self.writer, index)
    }
}
