//! .utoc directory index

use std::io::{Read, Seek, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use unreal_helpers::{UnrealReadExt, UnrealWriteExt};

use crate::error::Error;

/// IoStore .utoc directory index entry
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreDirectoryIndexEntry {
    /// Name
    pub name: u32,
    /// First child entry
    pub first_child_entry: u32,
    /// Next sibling entry
    pub next_sibling_entry: u32,
    /// First file entry
    pub first_file_entry: u32,
}

impl IoStoreDirectoryIndexEntry {
    /// Read `IoStoreDirectoryIndexEntry` from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let name = reader.read_u32::<LE>()?;
        let first_child_entry = reader.read_u32::<LE>()?;
        let next_sibling_entry = reader.read_u32::<LE>()?;
        let first_file_entry = reader.read_u32::<LE>()?;

        Ok(IoStoreDirectoryIndexEntry {
            name,
            first_child_entry,
            next_sibling_entry,
            first_file_entry,
        })
    }

    /// Write IoStoreDirectoryIndexEntry` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32::<LE>(self.name)?;
        writer.write_u32::<LE>(self.first_child_entry)?;
        writer.write_u32::<LE>(self.next_sibling_entry)?;
        writer.write_u32::<LE>(self.first_file_entry)?;

        Ok(())
    }
}

/// IoStore .utoc file index entry
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreFileIndexEntry {
    /// Name
    pub name: u32,
    /// Next file entry
    pub next_file_entry: u32,
    /// User data
    pub user_data: u32,
}

impl IoStoreFileIndexEntry {
    /// Read `IoStoreFileIndexEntry` from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let name = reader.read_u32::<LE>()?;
        let next_file_entry = reader.read_u32::<LE>()?;
        let user_data = reader.read_u32::<LE>()?;

        Ok(IoStoreFileIndexEntry {
            name,
            next_file_entry,
            user_data,
        })
    }

    /// Write `IoStoreFileIndexEntry` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_u32::<LE>(self.name)?;
        writer.write_u32::<LE>(self.next_file_entry)?;
        writer.write_u32::<LE>(self.user_data)?;

        Ok(())
    }
}

/// IoStore .utoc directory index
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IoStoreDirectoryIndex {
    /// Mount point
    pub mount_point: Option<String>,
    /// Directory entries
    pub directory_entries: Vec<IoStoreDirectoryIndexEntry>,
    /// File entries
    pub file_entries: Vec<IoStoreFileIndexEntry>,
    /// String table
    pub string_table: Vec<Option<String>>,
}

impl IoStoreDirectoryIndex {
    /// Root file index
    pub const ROOT_INDEX: u32 = 0;
    /// Invalid file index
    pub const INVALID_INDEX: u32 = u32::MAX;

    /// Read `IoStoreDirectoryIndex` from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, Error> {
        let mount_point = reader.read_fstring()?;

        let directory_entries_count = reader.read_i32::<LE>()?;
        let mut directory_entries = Vec::with_capacity(directory_entries_count as usize);
        for _ in 0..directory_entries_count {
            directory_entries.push(IoStoreDirectoryIndexEntry::read(reader)?);
        }

        let file_entries_count = reader.read_i32::<LE>()?;
        let mut file_entries = Vec::with_capacity(file_entries_count as usize);
        for _ in 0..file_entries_count {
            file_entries.push(IoStoreFileIndexEntry::read(reader)?);
        }

        let string_table_count = reader.read_i32::<LE>()?;
        let mut string_table = Vec::with_capacity(string_table_count as usize);
        for _ in 0..string_table_count {
            string_table.push(reader.read_fstring()?);
        }

        Ok(IoStoreDirectoryIndex {
            mount_point,
            directory_entries,
            file_entries,
            string_table,
        })
    }

    /// Write `IoStoreDirectoryIndex` to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_fstring(self.mount_point.as_deref())?;

        writer.write_i32::<LE>(self.directory_entries.len() as i32)?;
        for entry in &self.directory_entries {
            entry.write(writer)?;
        }

        writer.write_i32::<LE>(self.file_entries.len() as i32)?;
        for entry in &self.file_entries {
            entry.write(writer)?;
        }

        writer.write_i32::<LE>(self.string_table.len() as i32)?;
        for string in &self.string_table {
            writer.write_fstring(string.as_deref())?;
        }

        Ok(())
    }

    /// Iterate every item in the directory index
    pub fn iter(&self, starting_index: u32, mut f: impl FnMut(u32, String)) {
        self.iter_impl(starting_index, String::default(), &mut f);
    }

    fn iter_impl(
        &self,
        starting_index: u32,
        accumulated_path: String,
        f: &mut impl FnMut(u32, String),
    ) {
        let root_entry = &self.directory_entries[starting_index as usize];

        let mut file = root_entry.first_file_entry;
        while file != Self::INVALID_INDEX {
            let file_entry = &self.file_entries[file as usize];

            let toc_entry_index = file_entry.user_data;

            let name = self.string_table[file_entry.name as usize]
                .clone()
                .unwrap_or_default();
            let path = accumulated_path.clone() + "/" + &name;

            f(toc_entry_index, path);

            file = file_entry.next_file_entry;
        }

        let mut dir = root_entry.first_child_entry;
        while dir != Self::INVALID_INDEX {
            let directory_entry = &self.directory_entries[dir as usize];

            let name = self.string_table[directory_entry.name as usize]
                .clone()
                .unwrap_or_default();

            let path = accumulated_path.clone() + "/" + &name;

            self.iter_impl(dir, path, f);

            dir = directory_entry.next_sibling_entry;
        }
    }
}
