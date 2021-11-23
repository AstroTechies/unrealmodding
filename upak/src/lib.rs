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
use std::io::{BufReader, Error};

use num_enum::{IntoPrimitive, TryFromPrimitive};

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
        Ok(())
    }
}
