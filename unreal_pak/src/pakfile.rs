//! PakFile data structure for reading large pak files

use std::collections::BTreeMap;
use std::io::{BufReader, BufWriter, Read, Seek, Write};

use crate::entry::{read_entry, write_entry};
use crate::error::PakError;
use crate::header::Header;
use crate::index::{Footer, Index};
use crate::pakversion::PakVersion;
use crate::CompressionMethod;

/// An Unreal pak file with it's data kept on disk and only read on demand.
/// Good for working with very large files, but it has restrictions when it
/// comes to writing files. For a more flexible alternative see <insert doc link> PakMemory
#[derive(Debug)]
pub struct PakFile<'data, R, W>
where
    &'data R: Read + Seek,
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
    reader: Option<BufReader<&'data R>>,
    writer: Option<BufWriter<&'data W>>,
}

impl<'data, R, W> PakFile<'data, R, W>
where
    &'data R: Read + Seek,
    &'data W: Write + Seek,
{
    /// Creates a new `PakFile` configured to read files.
    pub fn reader(reader: &'data R) -> Self {
        Self {
            pak_version: PakVersion::PakFileVersionInvalid,
            mount_point: "".to_owned(),
            compression: CompressionMethod::Unknown,
            block_size: 0x010000,
            entries: BTreeMap::new(),
            reader: Some(BufReader::new(reader)),
            writer: None,
        }
    }

    /// Creates a new `PakFile` configured to write files.
    pub fn writer(
        writer: &'data W,
        pak_version: PakVersion,
        compression: CompressionMethod,
    ) -> Self {
        Self {
            pak_version,
            mount_point: "../../../".to_owned(),
            compression,
            block_size: 0x010000,
            entries: BTreeMap::new(),
            reader: None,
            writer: Some(BufWriter::new(writer)),
        }
    }

    /// Load the entry info contained in the footer into memory to start reading individual entries.
    pub fn load_index(&mut self) -> Result<(), PakError> {
        if let Some(ref mut reader) = &mut self.reader {
            let index = Index::read(reader)?;

            self.pak_version = index.footer.pak_version;
            self.mount_point = index.mount_point.clone();
            //? maybe also store compression somehow?

            for (name, header) in index.entries {
                self.entries.insert(name, header);
            }

            Ok(())
        } else {
            Err(PakError::configuration_invalid())
        }
    }

    /// Returns the names of all entries which have been found.
    pub fn get_entry_names(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    /// Reads an entry from the pak on disk into memory and returns it's data.
    pub fn read_entry(&mut self, name: &String) -> Result<Vec<u8>, PakError> {
        if let Some(ref mut reader) = &mut self.reader {
            let header = self
                .entries
                .get(name)
                .ok_or_else(|| PakError::entry_not_found(name.clone()))?;
            read_entry(reader, self.pak_version, header.offset)
        } else {
            Err(PakError::configuration_invalid())
        }
    }

    /// Writes the given data into the pak file on disk.
    /// Writes should happen in an aplphabetical order.
    pub fn write_entry(&mut self, name: &String, data: &Vec<u8>) -> Result<(), PakError> {
        if let Some(ref mut writer) = &mut self.writer {
            if self.entries.contains_key(name) {
                return Err(PakError::double_write(name.clone()));
            }

            let header = write_entry(
                writer,
                self.pak_version,
                data,
                self.compression,
                self.block_size,
            )?;
            self.entries.insert(name.clone(), header);

            Ok(())
        } else {
            Err(PakError::configuration_invalid())
        }
    }

    /// Finish writing the pak file by writing index and footer
    pub fn finish_write(mut self) -> Result<(), PakError> {
        if let Some(ref mut writer) = &mut self.writer {
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

            Index::write(writer, index)
        } else {
            Err(PakError::configuration_invalid())
        }
    }
}
