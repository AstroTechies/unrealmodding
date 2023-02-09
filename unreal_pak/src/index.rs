use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::buf_ext::{BufReaderExt, BufWriterExt};
use crate::compression::CompressionMethods;
use crate::error::PakError;
use crate::header::Header;
use crate::pakversion::PakVersion;
use crate::{hash, PAK_MAGIC};

#[derive(Debug)]
pub(crate) struct Index {
    pub mount_point: String,
    pub entries: Vec<(String, Header)>,
    pub footer: Footer,
}

impl Index {
    pub(crate) fn read<R: Read + Seek>(mut reader: &mut R) -> Result<Self, PakError> {
        let footer = Footer::read(&mut reader)?;

        reader.seek(SeekFrom::Start(footer.index_offset))?;

        let mount_point = reader.read_fstring()?.unwrap_or_default();

        let entry_count = reader.read_u32::<LittleEndian>()?;
        let mut entries = Vec::with_capacity(entry_count as usize);

        if footer.pak_version < PakVersion::PathHashIndex {
            for _ in 0..entry_count {
                let file_name = reader.read_fstring()?.ok_or_else(PakError::pak_invalid)?;

                entries.push((file_name, Header::read(reader, footer.pak_version)?));
            }
        } else {
            return Err(PakError::pak_version_unsupported(footer.pak_version));
        }

        Ok(Index {
            mount_point,
            entries,
            footer,
        })
    }

    pub(crate) fn write<W: Write + Seek>(writer: &mut W, mut index: Self) -> Result<(), PakError> {
        let index_offset = writer.stream_position()?;

        let mut index_writer = Cursor::new(Vec::new());

        index_writer.write_fstring(Some(&index.mount_point))?;

        index_writer.write_u32::<LittleEndian>(index.entries.len() as u32)?;

        if index.footer.pak_version < PakVersion::PathHashIndex {
            for (name, header) in index.entries {
                index_writer.write_fstring(Some(name.as_str()))?;
                Header::write(&mut index_writer, index.footer.pak_version, &header)?;
            }
        } else {
            return Err(PakError::pak_version_unsupported(index.footer.pak_version));
        }

        let index_data = index_writer.into_inner();
        index.footer.index_offset = index_offset;
        index.footer.index_size = index_data.len() as u64;

        index.footer.index_hash = hash(&index_data);

        writer.write_all(&index_data)?;

        Footer::write(writer, index.footer)?;

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct Footer {
    pub pak_version: PakVersion,
    pub index_offset: u64,
    pub index_size: u64,
    pub index_hash: [u8; 20],
    pub compression_methods: CompressionMethods,
    pub index_encrypted: Option<bool>,
    pub encryption_key_guid: Option<[u8; 0x10]>,
}

impl Footer {
    pub(crate) fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, PakError> {
        // magic offset (from bottom) can only be 0x2C (v2-v7), 0xAC (v8a), 0xCC (v8b,v11), 0xCD (v9)
        let possible_offsets = vec![-0x2C, -0xAC, -0xCC, -0xCD];

        let mut magic_offset = None;
        for offset in possible_offsets {
            reader.seek(SeekFrom::End(offset))?;
            if reader.read_u32::<BigEndian>()? == PAK_MAGIC {
                magic_offset = Some(offset);
            }
        }
        let magic_offset = magic_offset.ok_or_else(PakError::pak_invalid)?;

        // seek to file version
        reader.seek(SeekFrom::End(magic_offset + 4))?;

        let mut pak_version = PakVersion::from_num(reader.read_u32::<LittleEndian>()?);
        if magic_offset == -0xAC {
            pak_version.set_subversion();
        }

        let index_offset = reader.read_u64::<LittleEndian>()?;
        let index_size = reader.read_u64::<LittleEndian>()?;

        let mut index_hash = [0u8; 20];
        reader.read_exact(&mut index_hash)?;

        // if version 9 skip frozen index byte
        if pak_version == PakVersion::FrozenIndex {
            reader.seek(SeekFrom::Current(1))?;
        }

        let compression_methods = if pak_version >= PakVersion::FnameBasedCompressionMethod {
            CompressionMethods::from_reader(reader)?
        } else {
            CompressionMethods::default()
        };

        // index_encrypted is one byte before magic
        let mut index_encrypted = None;
        if pak_version >= PakVersion::IndexEncryption {
            reader.seek(SeekFrom::End(magic_offset - 1))?;
            index_encrypted = Some(reader.read_u8()? != 0);
        }

        // encryption key guid is 0x10 bytes before index_encrypted flag
        let mut encryption_key_guid = None;
        if pak_version >= PakVersion::EncryptionKeyGuid {
            reader.seek(SeekFrom::End(magic_offset - 0x11))?;
            let mut buf = [0u8; 0x10];
            reader.read_exact(&mut buf)?;
            encryption_key_guid = Some(buf);
        }

        Ok(Footer {
            pak_version,
            index_offset,
            index_size,
            index_hash,
            compression_methods,
            index_encrypted,
            encryption_key_guid,
        })
    }

    pub(crate) fn write<W: Write>(writer: &mut W, footer: Self) -> Result<(), PakError> {
        // write encryption key guid first
        if footer.pak_version >= PakVersion::EncryptionKeyGuid {
            if let Some(encryption_key_guid) = footer.encryption_key_guid {
                writer.write_all(&encryption_key_guid)?;
            }
        }

        // write index_encrypted
        if footer.pak_version >= PakVersion::IndexEncryption {
            writer.write_u8(u8::from(footer.index_encrypted.unwrap_or_default()))?;
        }

        // write magic and pak version
        writer.write_u32::<BigEndian>(PAK_MAGIC)?;
        writer.write_u32::<LittleEndian>(footer.pak_version.to_num())?;

        // write index offset and length
        writer.write_u64::<LittleEndian>(footer.index_offset)?;
        writer.write_u64::<LittleEndian>(footer.index_size)?;

        // write hash
        writer.write_all(&footer.index_hash)?;

        // frozen index
        if footer.pak_version == PakVersion::FrozenIndex {
            writer.write_u8(0)?;
        }

        // compression methods
        if footer.pak_version >= PakVersion::FnameBasedCompressionMethod {
            writer.write_all(footer.compression_methods.as_bytes().as_slice())?;
        }

        Ok(())
    }
}
