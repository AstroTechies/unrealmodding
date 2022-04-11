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
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use sha1::{Digest, Sha1};

mod error;
use error::UpakError;

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

#[derive(Debug)]
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

    pub fn load_records(&mut self) -> Result<(), UpakError> {
        // seek to header at the bottom of the file
        self.reader.seek(SeekFrom::End(-204))?;

        // read and check magic bytes
        if self.reader.read_u32::<BigEndian>()? != UE4_PAK_MAGIC {
            return Err(UpakError::invalid_pak_file());
        }

        // read and check file version
        let file_version = self.reader.read_u32::<LittleEndian>()?;
        if file_version != 8 {
            return Err(UpakError::unsupported_pak_version(file_version));
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
                    Err(_) => {
                        return Err(UpakError::invalid_pak_file());
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
                let record_compression_method = match CompressionMethod::try_from_primitive(
                    self.reader.read_u32::<LittleEndian>()?,
                ) {
                    Ok(compression_method) => compression_method,
                    Err(_) => CompressionMethod::Unknown,
                };

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
            return Err(UpakError::unsupported_pak_version(self.file_version));
        }

        Ok(())
    }

    pub fn read_record(&mut self, record_name: &String) -> Result<Vec<u8>, UpakError> {
        // find record
        let record = match self.records.get(record_name) {
            Some(record) => record,
            None => {
                return Err(UpakError::record_not_found(record_name.clone()));
            }
        };

        if self.file_version == 8 {
            if record.compression_method == CompressionMethod::None {
                // seek to data
                self.reader.seek(SeekFrom::Start(record.offset + 0x35))?;

                let mut buf = vec![0u8; record.decompressed_size as usize];
                self.reader.read_exact(&mut buf)?;

                return Ok(buf);
            } else if record.compression_method == CompressionMethod::Zlib {
                // skip unimportant data
                self.reader.seek(SeekFrom::Start(record.offset + 0x30))?;

                // read blocks
                let mut blocks = Vec::new();

                // read block count
                let block_count = self.reader.read_u32::<LittleEndian>()?;

                // read blocks
                for _i in 0..block_count {
                    // read block start
                    let block_start = self.reader.read_u64::<LittleEndian>()?;
                    // read block end
                    let block_end = self.reader.read_u64::<LittleEndian>()?;

                    // add block
                    blocks.push(Block {
                        start: block_start,
                        size: block_end - block_start,
                    });
                }

                // is_encrypted byte
                let mut buf1 = [0u8; 1];
                self.reader.read_exact(&mut buf1)?;
                let is_encrypted = buf1[0] != 0;
                // if is_encrypted return error
                if is_encrypted {
                    return Err(UpakError::enrcryption_unsupported());
                }

                // read block size
                let block_size = self.reader.read_u32::<LittleEndian>()?;

                // calculate header size and sub from blocks
                let header_size = self.reader.stream_position()? - record.offset;
                blocks = blocks
                    .iter()
                    .map(|block| Block {
                        start: block.start - header_size,
                        size: block.size,
                    })
                    .collect();

                // allocate buffers
                let mut record_data = vec![0u8; record.decompressed_size as usize];
                let mut compressed_data = vec![0u8; record.size as usize];

                // read all record compressed data (we are already at the start of the first block)
                self.reader.read_exact(&mut compressed_data)?;

                for i in 0..blocks.len() {
                    let block = &blocks[i];

                    let block_compressed_data =
                        &compressed_data[block.start as usize..(block.start + block.size) as usize];

                    // decompress block
                    let mut decoder = ZlibDecoder::new(&block_compressed_data[..]);

                    // read decompressed data
                    let decompressed_start = block_size as usize * i;
                    let mut decompressed_end = block_size as usize * (i + 1);
                    if decompressed_end > record.decompressed_size as usize {
                        decompressed_end = record.decompressed_size as usize;
                    }

                    decoder.read_exact(&mut record_data[decompressed_start..decompressed_end])?;
                }

                return Ok(record_data);
            } else {
                return Err(UpakError::unsupported_compression(
                    record.compression_method,
                ));
            }
        } else {
            return Err(UpakError::unsupported_pak_version(self.file_version));
        }
    }

    pub fn init_empty(&mut self, file_version: u32) -> Result<(), UpakError> {
        if file_version == 8 {
            self.file_version = file_version;
            self.mount_point = b"../../../\0".iter().map(|&x| x).collect::<Vec<u8>>();
            self.block_size = 0x10000;
            self.records.clear();
        } else {
            return Err(UpakError::unsupported_pak_version(file_version));
        }

        Ok(())
    }

    pub fn write_record(
        &mut self,
        record_name: &String,
        record_data: &Vec<u8>,
        compression_method: &CompressionMethod,
    ) -> Result<(), UpakError> {
        let mut record = PakRecord {
            file_version: self.file_version,
            offset: 0,
            size: 0,
            decompressed_size: record_data.len() as u64,
            compression_method: compression_method.clone(),
            compression_blocks: Vec::new(),
            hash: Vec::new(),
        };

        if self.file_version == 8 {
            // allocate buffer for final data
            let mut compressed_data: Vec<u8> = Vec::new();

            if compression_method == &CompressionMethod::None || record_data.len() < 10 {
                // simply clone data when no compression is used
                compressed_data = record_data.clone();
                record.compression_method = CompressionMethod::None;
            } else if compression_method == &CompressionMethod::Zlib {
                // split into blocks
                let num_blocks = (record_data.len() as f64 / self.block_size as f64).ceil() as u32;

                for i in 0..num_blocks {
                    let block_start = i as u64 * self.block_size as u64;
                    let mut block_end = (i + 1) as u64 * self.block_size as u64;
                    if block_end > record_data.len() as u64 {
                        block_end = record_data.len() as u64;
                    }

                    let block_uncompressed_data =
                        &record_data[block_start as usize..block_end as usize];

                    let length_before = compressed_data.len();

                    // compress data
                    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(&block_uncompressed_data)?;
                    let block_compressed_data = encoder.finish()?;

                    compressed_data.extend_from_slice(&block_compressed_data);

                    record.compression_blocks.push(SimpleBlock {
                        start: length_before as u64,
                        end: compressed_data.len() as u64,
                    });
                }
            }

            // set size
            record.size = compressed_data.len() as u64;

            // compute sha1 hash
            let mut hasher = Sha1::new();
            hasher.update(&compressed_data);
            record.hash = hasher.finalize().to_vec();

            // store offset
            let record_offset = self.reader.stream_position()?;

            // generate and write header
            self.reader
                .get_mut()
                .write_all(&Self::generate_header(&record, self.block_size))?;

            // write data
            self.reader.get_mut().write_all(&compressed_data)?;

            // update record with offset
            record.offset = record_offset;
        } else {
            return Err(UpakError::unsupported_pak_version(self.file_version));
        }

        // add record
        self.records.insert(record_name.to_string(), record);

        Ok(())
    }

    pub fn write_index_and_footer(&mut self) -> Result<(), UpakError> {
        if self.file_version == 8 {
            let file = self.reader.get_mut();
            let index_offset = file.stream_position()?;

            // generate index
            let mut index_data: Vec<u8> = Vec::new();

            // write mount point length
            let mount_point_length = (self.mount_point.len() as u32).to_le_bytes();
            index_data.extend(&mount_point_length);
            // write mount point
            index_data.extend(&self.mount_point);

            // write record count
            let record_count = (self.records.len() as u32).to_le_bytes();
            index_data.extend(&record_count);

            // write records
            let mut record_names = self.records.keys().collect::<Vec<&String>>();
            record_names.sort();
            for record_name in record_names {
                let record = self.records.get(record_name).unwrap();

                // write record name length
                let record_name_length = (record_name.len() as u32 + 1).to_le_bytes();
                index_data.extend(&record_name_length);
                // write record name
                index_data.extend(record_name.as_bytes());
                // write extra null byte
                index_data.extend(&[0x00]);

                // write record
                index_data.extend(&Self::generate_header(&record, self.block_size));
            }

            // write index to file
            file.write_all(&index_data)?;

            let index_size = file.stream_position()? - index_offset;

            // write footer
            // write 16 empty bytes
            file.write_all(&[0; 17])?;

            // write magic
            file.write_all(&UE4_PAK_MAGIC.to_be_bytes())?;

            // write file version
            file.write_all(&self.file_version.to_le_bytes())?;

            // write index offset
            file.write_all(&index_offset.to_le_bytes())?;

            // write index size
            file.write_all(&index_size.to_le_bytes())?;

            // write index hash
            let mut hasher = Sha1::new();
            hasher.update(&index_data);
            file.write_all(&hasher.finalize().to_vec())?;

            // write "Zlib" text
            file.write_all(b"Zlib")?;

            // write 0x9C empty bytes
            file.write_all(&[0; 0x9C])?;
        } else {
            return Err(UpakError::unsupported_pak_version(self.file_version));
        }

        Ok(())
    }

    fn generate_header(record: &PakRecord, block_size: u32) -> Vec<u8> {
        // calculate header size
        let header_size = 8
            + 8
            + 8
            + 4
            + 20
            + if record.compression_method != CompressionMethod::None {
                record.compression_blocks.len() as u64 * 16 + 4
            } else {
                0
            }
            + 1
            + 4;

        // allocate buffer
        let mut header = vec![0u8; header_size as usize];

        // write offset
        header[0..8].copy_from_slice(&record.offset.to_le_bytes());

        // write size
        header[8..16].copy_from_slice(&record.size.to_le_bytes());

        // write decompressed size
        header[16..24].copy_from_slice(&record.decompressed_size.to_le_bytes());

        // write compression method
        let compression_method: u32 = record.compression_method.into();
        header[24..28].copy_from_slice(&compression_method.to_le_bytes());

        // write sha1 hash
        header[28..48].copy_from_slice(&record.hash);

        // write compression blocks
        if record.compression_method != CompressionMethod::None {
            // write block count
            header[48..52].copy_from_slice(&(record.compression_blocks.len() as u32).to_le_bytes());

            for i in 0..record.compression_blocks.len() {
                let block = &record.compression_blocks[i];

                // write block start
                header[52 + i * 16..52 + i * 16 + 8]
                    .copy_from_slice(&(block.start + header_size).to_le_bytes());

                // write block end
                header[52 + i * 16 + 8..52 + i * 16 + 16]
                    .copy_from_slice(&(block.end + header_size).to_le_bytes());
            }
        }

        // write is encrypted flag
        header[header_size as usize - 5] = 0u8;

        // write block size
        let mut use_block_size = block_size;
        if (use_block_size as u64) > record.decompressed_size {
            use_block_size = record.decompressed_size as u32;
        }
        if record.compression_method == CompressionMethod::None {
            use_block_size = 0;
        }
        header[header_size as usize - 4..].copy_from_slice(&use_block_size.to_le_bytes());

        header
    }
}
