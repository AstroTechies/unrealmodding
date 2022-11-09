use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use sha1::{Digest, Sha1};

use crate::buf_ext::{BufReaderExt, BufWriterExt};
use crate::error::PakError;
use crate::header::Header;
use crate::pakversion::PakVersion;
use crate::PAK_MAGIC;

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

        if footer.pak_version < PakVersion::PakFileVersionPathHashIndex {
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

        if index.footer.pak_version < PakVersion::PakFileVersionPathHashIndex {
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

        let mut hasher = Sha1::new();
        hasher.update(&index_data);
        // sha1 always outputs 20 bytes
        index.footer.index_hash = hasher.finalize().to_vec().try_into().unwrap();

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
    pub index_encrypted: Option<bool>,
    pub encryption_key_guid: Option<[u8; 0x10]>,
}

impl Footer {
    pub(crate) fn read<R: Read + Seek>(reader: &mut R) -> Result<Self, PakError> {
        // magic offset can only be 0x2C (2-7), 0xCC (8,11), 0xCD (9)
        let possible_offsets = vec![-0x2C, -0xCC, -0xCD];

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

        let pak_version = PakVersion::try_from(reader.read_i32::<LittleEndian>()?)
            .map_err(|_| PakError::pak_invalid())?;

        let index_offset = reader.read_u64::<LittleEndian>()?;
        let index_size = reader.read_u64::<LittleEndian>()?;

        let mut index_hash = [0u8; 20];
        reader.read_exact(&mut index_hash)?;

        // index_encrypted is one byte before magic
        let mut index_encrypted = None;
        if pak_version >= PakVersion::PakFileVersionIndexEncryption {
            reader.seek(SeekFrom::End(magic_offset - 1))?;
            index_encrypted = Some(reader.read_u8()? != 0);
        }

        // encryption key guid is 0x10 bytes before index_encrypted flag
        let mut encryption_key_guid = None;
        if pak_version >= PakVersion::PakFileVersionEncryptionKeyGuid {
            reader.seek(SeekFrom::End(magic_offset - 0x11))?;
            let mut buf = [0u8; 0x10];
            reader.read_exact(&mut buf)?;
            encryption_key_guid = Some(buf);
        }

        // TODO: read frozen index and compression FStrings (5 * 0x20)

        Ok(Footer {
            pak_version,
            index_offset,
            index_size,
            index_hash,
            index_encrypted,
            encryption_key_guid,
        })
    }

    pub(crate) fn write<W: Write>(writer: &mut W, footer: Self) -> Result<(), PakError> {
        // write encryption key guid first
        if footer.pak_version >= PakVersion::PakFileVersionEncryptionKeyGuid {
            if let Some(encryption_key_guid) = footer.encryption_key_guid {
                writer.write_all(&encryption_key_guid)?;
            }
        }

        // write index_encrypted
        if footer.pak_version >= PakVersion::PakFileVersionIndexEncryption {
            writer.write_u8(if footer.index_encrypted.unwrap_or_default() {
                1
            } else {
                0
            })?;
        }

        // write magic and pak version
        writer.write_u32::<BigEndian>(PAK_MAGIC)?;
        writer.write_i32::<LittleEndian>(footer.pak_version.into())?;

        // write index offset and length
        writer.write_u64::<LittleEndian>(footer.index_offset)?;
        writer.write_u64::<LittleEndian>(footer.index_size)?;

        // write hash
        writer.write_all(&footer.index_hash)?;

        // frozen index
        if footer.pak_version == PakVersion::PakFileVersionFrozenIndex {
            writer.write_u8(0)?;
        }

        // compression methods
        const COMPRESSION_NAME: &[u8] = b"Zlib";
        writer.write_all(COMPRESSION_NAME)?;
        // filler bytes to fill space intended for 5 * 0x20 compression names
        writer.write_all(&[0u8; 5 * 0x20 - COMPRESSION_NAME.len()])?;

        Ok(())
    }
}
