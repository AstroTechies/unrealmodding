/*
Unreal pak format
File parts:
    - recordss
        - header
        - data
    - index
        - records entries
            - record name
            - record header
    - footer

header:
    - u64 offset (when infront of record empty)
    - u64 size
    - u64 size decompressed
    - u32 compression method
    - 20 bytes sha1 hash
    - compression block data (only when compression method is not 0)
        - u32 number of blocks
        - blocks
            - u64 block start
            - u64 block end
    - u8 is encrypted flag
    - u32 block size
*/

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

use num_enum::{IntoPrimitive, TryFromPrimitive};

const UE4_PAK_MAGIC: u32 = u32::from_be_bytes([0xe1, 0x12, 0x6f, 0x5a]);

#[derive(PartialEq, Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum CompressionMethod {
    None = 0,
    Zlib = 1,
    BiasMemory = 2,
    BiasSpeed = 3,
    Unknown = 255,
}

pub struct PakFile<'file> {
    pub file_version: u32,
    pub mount_point: Vec<u8>,
    pub block_size: u32,
    pub records: HashMap<String, PakRecord>,
    reader: BufReader<&'file File>,
}

#[derive(Debug, Clone)]
pub struct PakRecord {
    pub file_version: u32,
    pub offset: u64,
    pub size: u64,
    pub decompressed_size: u64,
    pub compression_method: CompressionMethod,
    compression_blocks: Vec<SimpleBlock>,
    hash: Vec<u8>,
}

#[derive(Debug, Clone)]
struct Block {
    pub start: u64,
    pub size: u64,
}

#[derive(Debug, Clone)]
struct SimpleBlock {
    pub start: u64,
    pub end: u64,
}

impl<'file> PakFile<'file> {
    pub fn new(file: &'file File) -> Self {
        let reader = BufReader::new(file);
        PakFile {
            file_version: 0,
            mount_point: Vec::new(),
            block_size: 0,
            records: HashMap::new(),
            reader,
        }
    }

    pub fn load_records(&mut self) -> Result<(), Error> {
        // seek to header at the bottom of the file
        self.reader.seek(SeekFrom::End(-204))?;

        // read and check magic bytes
        if self.reader.read_u32::<BigEndian>()? != UE4_PAK_MAGIC {
            return Err(Error::new(ErrorKind::Other, "File is not a valid pak file"));
        }

        // read and check file version
        let file_version = self.reader.read_u32::<LittleEndian>()?;
        if file_version != 8 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unsupported file version {}", file_version),
            ));
        }
        self.file_version = file_version;

        if self.file_version == 8 {
            // read index offset
            let offset = self.reader.read_u64::<LittleEndian>()?;
            self.reader.seek(SeekFrom::Start(offset))?;

            // read mount point
            let mount_point_len = self.reader.read_u32::<LittleEndian>()?;
            let mut mount_point_buf = vec![0u8; mount_point_len as usize];
            self.reader.read_exact(&mut mount_point_buf)?;
            self.mount_point = mount_point_buf;

            // read record count
            let record_count = self.reader.read_u32::<LittleEndian>()?;

            // read records data
            for _i in 0..record_count {
                // read record name
                let record_name_len = self.reader.read_u32::<LittleEndian>()?;
                let mut record_name_buf = vec![0u8; record_name_len as usize];
                self.reader.read_exact(&mut record_name_buf)?;
                let mut record_name = match String::from_utf8(record_name_buf) {
                    Ok(record_name) => record_name,
                    Err(err) => {
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("Record name is not valid utf8: {:?}", err),
                        ));
                    }
                };
                record_name.pop();

                // read record offset
                let record_offset = self.reader.read_u64::<LittleEndian>()?;
                // read record size
                let record_size = self.reader.read_u64::<LittleEndian>()?;
                // read record decompressed size
                let record_decompressed_size = self.reader.read_u64::<LittleEndian>()?;
                // read record compression method
                let record_compression_method =
                    CompressionMethod::try_from_primitive(self.reader.read_u32::<LittleEndian>()?).unwrap();

                // seek over hash
                self.reader.seek_relative(20)?;

                if record_compression_method != CompressionMethod::None {
                    // read block count
                    let block_count = self.reader.read_u32::<LittleEndian>()?;

                    // seek over block data
                    self.reader
                        .seek(SeekFrom::Current(16 * block_count as i64))?;
                }

                // skip is_encrypted and block size
                self.reader.seek(SeekFrom::Current(5))?;

                // add record
                self.records.insert(
                    record_name,
                    PakRecord {
                        file_version: self.file_version,
                        offset: record_offset,
                        size: record_size,
                        decompressed_size: record_decompressed_size,
                        compression_method: record_compression_method,
                        compression_blocks: Vec::new(),
                        hash: Vec::new(),
                    },
                );
            }
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unsupported file version {}", self.file_version),
            ));
        }

        Ok(())
    }
}
