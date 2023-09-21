//! PakFile data structure for writing large pak files

use std::collections::BTreeMap;
use std::io::{Seek, Write};

use crate::compression::CompressionMethods;
use crate::entry::write_entry;
use crate::error::PakError;
use crate::header::Header;
use crate::index::{random_path_hash_seed, Footer, Index};
use crate::pakversion::PakVersion;

/// An Unreal pak file writer which allows incrementally writing data.
/// Good for working with very large files, but it has restrictions when it
/// comes to writing files. For a more flexible alternative see [`PakMemory`].
///
/// [`PakMemory`]: crate::pakmemory::PakMemory
#[derive(Debug)]
pub struct PakWriter<W>
where
    W: Write + Seek,
{
    /// Version of the pak file format this one is using
    pub pak_version: PakVersion,
    /// Mount point. Typically `../../../`.
    pub mount_point: String,
    /// Compression method preferred for this file
    compression: CompressionMethods,
    /// Compression block size
    pub block_size: u32,
    entries: BTreeMap<String, Header>,
    writer: W,
}

impl<W> PakWriter<W>
where
    W: Write + Seek,
{
    /// Creates a new `PakWriter` that writes to the provided writer.
    /// When using a writer that uses syscalls like a `File` it is recommended to wrap it in a
    /// [`std::io::BufWriter`] to avoid unnecessary syscalls.
    pub fn new(writer: W, pak_version: PakVersion) -> Self {
        Self {
            pak_version,
            mount_point: "../../../".to_owned(),
            compression: CompressionMethods::zlib(),
            block_size: 0x010000,
            entries: BTreeMap::new(),
            writer,
        }
    }

    /// Returns the names of all entries which have been found.
    pub fn get_entry_names(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    /// Writes the given data into the pak file on disk.
    /// Writes should happen in an aplphabetical order.
    /// Entries under 32 bytes are never compressed.
    pub fn write_entry(
        &mut self,
        name: &String,
        data: &Vec<u8>,
        compress: bool,
    ) -> Result<(), PakError> {
        if self.entries.contains_key(name) {
            return Err(PakError::double_write(name.clone()));
        }

        let header = write_entry(
            &mut self.writer,
            self.pak_version,
            data,
            compress,
            &self.compression,
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
            compression_methods: self.compression,
            index_encrypted: Some(false),
            encryption_key_guid: Some([0u8; 0x10]),
        };

        let index = Index {
            mount_point: self.mount_point,
            path_hash_seed: Some(random_path_hash_seed()),
            entries: self.entries.into_iter().collect::<Vec<_>>(),
            footer,
        };

        Index::write(&mut self.writer, index)
    }
}
