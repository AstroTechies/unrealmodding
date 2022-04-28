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
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::mem::size_of;

use buf_ext::{BufReaderExt, BufWriterExt};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use pakversion::PakVersion;
use sha1::{Digest, Sha1};

mod buf_ext;
pub mod error;
pub mod pakversion;
use error::UpakError;

const UE4_PAK_MAGIC: u32 = u32::from_be_bytes([0xe1, 0x12, 0x6f, 0x5a]);

#[derive(PartialEq, Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum CompressionMethod {
    None = 0,
    Zlib = 1,
    BiasMemory = 2,
    BiasSpeed = 3,
    Unknown = 255,
}

#[derive(Debug)]
pub struct PakFile<'data, R>
where
    &'data R: Read + Write + Seek,
{
    pub file_version: PakVersion,
    pub mount_point: Vec<u8>,
    pub block_size: u32,
    pub records: HashMap<String, PakRecord>,
    reader: Option<BufReader<&'data R>>,
    writer: Option<BufWriter<&'data R>>,
}

#[derive(Debug, Clone)]
pub struct PakRecord {
    pub file_name: String,
    pub offset: u64,
    pub compressed_size: u64,
    pub decompressed_size: u64,
    pub compression_method: CompressionMethod,
    pub compression_block_size: Option<u32>,
    pub flags: Option<u8>,
    pub hash: Vec<u8>,
    pub data: Option<Vec<u8>>,

    compression_blocks: Option<Vec<Block>>,
}

impl PakRecord {
    pub fn new(
        file_name: String,
        uncompressed_data: Vec<u8>,
        compression_method: CompressionMethod,
    ) -> Result<Self, UpakError> {
        let record = PakRecord {
            file_name,
            offset: 0,
            compressed_size: 0,
            decompressed_size: 0,
            compression_method,
            compression_block_size: None,
            flags: None,
            hash: Vec::new(),
            compression_blocks: None,
            data: Some(uncompressed_data),
        };
        Ok(record)
    }

    fn read_header<R>(reader: &mut R, file_version: PakVersion) -> Result<Self, UpakError>
    where
        R: Read + Seek,
    {
        let file_name = reader.read_string()?.ok_or(UpakError::invalid_pak_file())?;
        let offset = reader.read_u64::<LittleEndian>()?;
        let compressed_size = reader.read_u64::<LittleEndian>()?;
        let decompressed_size = reader.read_u64::<LittleEndian>()?;
        let compression_method =
            match CompressionMethod::try_from_primitive(reader.read_i32::<LittleEndian>()?) {
                Ok(compression_method) => compression_method,
                Err(_) => CompressionMethod::Unknown,
            };

        if file_version <= PakVersion::PakFileVersionInitial {
            let _timestamp = reader.read_u64::<LittleEndian>()?;
        }

        let mut hash = [0u8; 20];
        reader.read_exact(&mut hash)?;

        let mut compression_blocks = None;
        let mut flags = None;
        let mut compression_block_size = None;

        if file_version >= PakVersion::PakFileVersionCompressionEncryption {
            if compression_method != CompressionMethod::None {
                let block_count = reader.read_u32::<LittleEndian>()? as usize;
                let mut compression_blocks_inner = Vec::with_capacity(block_count);

                for _ in 0..block_count {
                    let start_offset = reader.read_u64::<LittleEndian>()?;
                    let end_offset = reader.read_u64::<LittleEndian>()?;
                    compression_blocks_inner.push(Block {
                        start: start_offset,
                        size: end_offset - start_offset,
                    });
                }
                compression_blocks = Some(compression_blocks_inner);
            }

            flags = Some(reader.read_u8()?);
            compression_block_size = Some(reader.read_u32::<LittleEndian>()?);
        }

        Ok(PakRecord {
            file_name,
            offset,
            compressed_size,
            decompressed_size,
            compression_method,
            compression_block_size,
            compression_blocks,
            flags,
            hash: hash.to_vec(),
            data: None,
        })
    }

    fn read_data<R>(&mut self, reader: &mut R, file_version: PakVersion) -> Result<(), UpakError>
    where
        R: Read + Seek,
    {
        reader.seek(SeekFrom::Start(self.offset))?;
        let data = match self.compression_method {
            CompressionMethod::None => {
                let mut buf = vec![0u8; self.decompressed_size as usize];
                reader.read_exact(&mut buf)?;
                Ok(buf)
            }
            CompressionMethod::Zlib => {
                let mut decompressed_data = Vec::new();

                let compression_blocks = self
                    .compression_blocks
                    .as_ref()
                    .ok_or(UpakError::invalid_record())?;
                for block in compression_blocks {
                    let offset = block.start;

                    if file_version >= PakVersion::PakFileVersionRelativeChunkOffsets {
                        reader.seek(SeekFrom::Start(self.offset + offset))?;
                    } else {
                        reader.seek(SeekFrom::Start(offset))?;
                    }

                    let mut compressed_data = vec![0u8; block.size as usize];
                    reader.read_exact(&mut compressed_data)?;
                    let decoder = ZlibDecoder::new(&compressed_data[..]);
                    let decompressed_block: Vec<u8> = decoder.bytes().map(|e| e.unwrap()).collect();

                    decompressed_data.extend(decompressed_block);
                }

                Ok(decompressed_data)
            }
            _ => Err(UpakError::unsupported_compression(self.compression_method)),
        }?;
        self.data = Some(data);
        Ok(())
    }

    fn write_header<W>(&self, writer: &mut W, include_name: bool) -> Result<(), UpakError>
    where
        W: Write + Seek,
    {
        if include_name {
            writer.write_string(Some(&self.file_name))?;
        }

        let begin = writer.stream_position()?;
        writer.write_u64::<LittleEndian>(self.offset)?;
        writer.write_u64::<LittleEndian>(self.compressed_size)?;
        writer.write_u64::<LittleEndian>(self.decompressed_size)?;
        writer.write_i32::<LittleEndian>(self.compression_method.into())?;

        //todo: <= handling
        writer.write_all(&self.hash)?;

        let mut max_block_size = 0;
        if self.compression_method != CompressionMethod::None {
            writer
                .write_u32::<LittleEndian>(self.compression_blocks.as_ref().unwrap().len() as u32)?;
            let size = writer.stream_position()? - begin;
            let compression_block_offset = size
                + size_of::<u32>() as u64
                + size_of::<u8>() as u64
                + ((size_of::<u64>() * 2 * self.compression_blocks.as_ref().unwrap().len()) as u64);

            let compression_blocks_iter = self.compression_blocks.as_ref().unwrap().iter();
            for compression_block in compression_blocks_iter {
                if max_block_size < compression_block.size {
                    max_block_size = compression_block.size;
                }
                writer.write_u64::<LittleEndian>(
                    compression_block.start + compression_block_offset,
                )?;
                writer.write_u64::<LittleEndian>(
                    compression_block.start + compression_block.size + compression_block_offset,
                )?;
            }
        }

        writer.write_u8(0)?; // is_encrypted
        writer.write_u32::<LittleEndian>(max_block_size as u32)?;

        Ok(())
    }

    fn write<W>(&mut self, writer: &mut W, block_size: u32) -> Result<(), UpakError>
    where
        W: Write + Seek,
    {
        self.offset = writer.stream_position()?;

        let mut compressed_data = Vec::new();
        let data = match self.compression_method {
            CompressionMethod::Zlib => {
                self.compression_blocks = Some(Vec::new());
                let num_blocks =
                    (self.data.as_ref().unwrap().len() as f64 / block_size as f64).ceil() as u32;

                for i in 0..num_blocks {
                    let block_start = i as u64 * block_size as u64;
                    let mut block_end = (i + 1) as u64 * block_size as u64;
                    if block_end > self.data.as_ref().unwrap().len() as u64 {
                        block_end = self.data.as_ref().unwrap().len() as u64;
                    }

                    let block_uncompressed_data =
                        &self.data.as_ref().unwrap()[block_start as usize..block_end as usize];

                    let begin = compressed_data.len() as u64;

                    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(&block_uncompressed_data)?;
                    let block_compressed_data = encoder.finish()?;
                    compressed_data.extend_from_slice(&block_compressed_data);

                    self.compression_blocks.as_mut().unwrap().push(Block {
                        start: begin,
                        size: block_compressed_data.len() as u64,
                    });
                }
                Ok(&compressed_data)
            }
            CompressionMethod::None => Ok(self.data.as_ref().unwrap()),
            _ => Err(UpakError::invalid_record()),
        }?;

        self.decompressed_size = self.data.as_ref().unwrap().len() as u64;
        self.compressed_size = data.len() as u64;

        let mut hasher = Sha1::new();
        hasher.update(&data);
        self.hash = hasher.finalize().to_vec();

        self.write_header(writer, false)?;
        writer.write_all(&data)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Block {
    pub start: u64,
    pub size: u64,
}

impl<'data, R> PakFile<'data, R>
where
    &'data R: Write + Read + Seek,
{
    pub fn reader(file_version: PakVersion, data: &'data R) -> Self {
        PakFile {
            file_version,
            mount_point: "../../../".as_bytes().to_vec(),
            block_size: 0x10000,
            records: HashMap::new(),
            reader: Some(BufReader::new(data)),
            writer: None,
        }
    }

    pub fn writer(file_version: PakVersion, writer: BufWriter<&'data R>) -> Self {
        PakFile {
            file_version,
            mount_point: "../../../".as_bytes().to_vec(),
            block_size: 0x10000,
            records: HashMap::new(),
            reader: None,
            writer: Some(writer),
        }
    }

    pub fn load_records(&mut self) -> Result<(), UpakError> {
        if self.reader.is_none() {
            return Err(UpakError::invalid_pak_file());
        }
        let mut reader = self.reader.as_mut().unwrap();

        reader.seek(SeekFrom::End(-221))?;
        let mut _encryption_key_guid = [0u8; 16];
        reader.read_exact(&mut _encryption_key_guid)?;

        let is_encrypted = reader.read_u8()?;
        if is_encrypted != 0 {
            return Err(UpakError::enrcryption_unsupported());
        }

        let magic = reader.read_u32::<BigEndian>()?;
        if magic != UE4_PAK_MAGIC {
            return Err(UpakError::invalid_pak_file());
        }

        let file_version = PakVersion::try_from(reader.read_i32::<LittleEndian>()?)
            .map_err(|_| UpakError::invalid_pak_file())?;
        let index_offset = reader.read_u64::<LittleEndian>()?;
        let _index_size = reader.read_u64::<LittleEndian>()?;

        reader.seek(SeekFrom::Start(index_offset))?;

        let _mount_point = reader.read_string()?;
        let record_count = reader.read_u32::<LittleEndian>()?;

        for _ in 0..record_count {
            let record = PakRecord::read_header(&mut reader, file_version)?;
            self.records.insert(record.file_name.clone(), record);
        }
        Ok(())
    }

    pub fn add_record(&mut self, record: PakRecord) -> Result<(), UpakError> {
        self.records.remove(&record.file_name);
        self.records.insert(record.file_name.clone(), record);
        Ok(())
    }

    pub fn get_record(&mut self, name: &String) -> Result<&PakRecord, UpakError> {
        let record = self
            .records
            .get_mut(name)
            .ok_or(UpakError::record_not_found(name.clone()))?;
        record.read_data(self.reader.as_mut().unwrap(), self.file_version)?;
        Ok(record)
    }

    pub fn write(&mut self) -> Result<(), UpakError> {
        if self.writer.is_none() {
            return Err(UpakError::invalid_pak_file());
        }

        let mut writer = self.writer.as_mut().unwrap();

        for (_, record) in &mut self.records {
            record.write(&mut writer, self.block_size)?;
        }

        let index_offset = writer.stream_position()?;

        let mut header_writer = Cursor::new(Vec::new());
        header_writer.write_string(Some("../../../"))?;
        header_writer.write_i32::<LittleEndian>(self.records.len() as i32)?;

        for (_, record) in &self.records {
            record.write_header(&mut header_writer, true)?;
        }
        header_writer.flush()?;
        writer.write_all(header_writer.get_ref())?;

        let index_length = writer.stream_position()? - index_offset;

        if self.file_version >= PakVersion::PakFileVersionEncryptionKeyGuid {
            writer.write_all(&[0u8; 16])?;
        }
        writer.write_u8(0)?; // is_encrypted
        writer.write_u32::<BigEndian>(UE4_PAK_MAGIC)?;

        let file_version: i32 = self.file_version.into();
        writer.write_i32::<LittleEndian>(file_version)?;
        writer.write_u64::<LittleEndian>(index_offset)?;
        writer.write_u64::<LittleEndian>(index_length)?;

        let header = header_writer.get_ref();
        let mut hasher = Sha1::new();
        hasher.update(&header);
        let hash = hasher.finalize().to_vec();
        writer.write_all(&hash)?;

        let compression_method = self
            .records
            .values()
            .next()
            .map(|e| e.compression_method)
            .unwrap_or(CompressionMethod::None);

        match compression_method {
            CompressionMethod::Zlib => {
                writer.write_all(b"Zlib")?;
                Ok(())
            }
            CompressionMethod::None => {
                writer.write_all(&[0u8; 4])?;
                Ok(())
            }
            _ => Err(UpakError::unsupported_compression(compression_method)),
        }?;

        writer.write_all(&[0u8; 0x9c])?;

        Ok(())
    }
}
