//! IoStore .ucas reader

use std::{
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
};

use aes::{
    cipher::{generic_array::GenericArray, KeyInit},
    Aes256,
};

use unreal_asset_base::{
    compression::{self, CompressionMethod},
    error::{Error, IoStoreError},
};

use crate::iostore::{
    align,
    encryption::{self, ENCRYPTION_ALIGN},
    flags::EIoContainerFlags,
    providers::IoStoreProvider,
    toc::IoStoreTocResource,
};

/// IoStore .ucas file reader
#[derive(Debug, Clone)]
pub struct IoStoreReader<R, P>
where
    R: Read + Seek,
    P: IoStoreProvider<R>,
{
    /// IoStore .utoc resource
    pub toc_resource: IoStoreTocResource,
    /// Contaioner provider
    provider: P,
    /// File name
    file_name: String,
    /// Aes encryption key
    aes: Option<Aes256>,

    /// Marker
    _marker: PhantomData<R>,
}

impl<R, P> IoStoreReader<R, P>
where
    R: Read + Seek,
    P: IoStoreProvider<R>,
{
    /// Create a new `IoStoreReader` instance
    pub fn new(
        provider: P,
        file_name: &str,
        toc_resource: IoStoreTocResource,
        encryption_key: Option<[u8; 32]>,
    ) -> Result<Self, Error> {
        if toc_resource
            .header
            .container_flags
            .contains(EIoContainerFlags::ENCRYPTED)
            && encryption_key.is_none()
        {
            return Err(IoStoreError::NoEncryptionKey.into());
        }

        let aes = encryption_key.map(|e| {
            let array = GenericArray::from(e);
            Aes256::new(&array)
        });

        Ok(IoStoreReader {
            toc_resource,
            provider,
            file_name: file_name.to_owned(),
            aes,
            _marker: PhantomData,
        })
    }

    /// Read data from this reader
    ///
    /// This reads the data from an IoStore container
    /// and decompresses/decrypts blocks as needed
    pub fn read_all(&self, offset: u64, buf: &mut [u8]) -> Result<(), Error> {
        let compression_block_size = self.toc_resource.header.compression_block_size as u64;

        let first_block_index = offset / compression_block_size;

        // this is a worst-case estimate because buf size is for decompressed data
        let last_block_end = align::align(offset + buf.len() as u64, compression_block_size);
        let last_block_index = (last_block_end - 1) / compression_block_size;

        let mut remaining_size = buf.len();

        let mut offset_in_block = offset % compression_block_size;

        for block_index in first_block_index..=last_block_index {
            let compression_block = &self.toc_resource.compression_blocks[block_index as usize];

            let partition_index =
                compression_block.offset / self.toc_resource.header.partition_size;
            let partition_offset =
                compression_block.offset % self.toc_resource.header.partition_size;

            let read_size =
                align::align(compression_block.compressed_size as u64, ENCRYPTION_ALIGN) as usize;

            let mut reader = self.create_partition_reader(partition_index)?;
            reader.seek(SeekFrom::Start(partition_offset))?;

            let mut data = vec![0u8; read_size];
            reader.read_exact(&mut data)?;

            // decryption
            if let Some(e) = &self.aes {
                encryption::decrypt(e, &mut data);
            };

            let compression_method = match compression_block.compression_method_index == 0 {
                true => CompressionMethod::None,
                false => self.toc_resource.compression_methods
                    [compression_block.compression_method_index as usize - 1]
                    .clone(),
            };

            let mut decompressed = vec![0u8; compression_block.decompressed_size as usize];
            compression::decompress(compression_method, &data, &mut decompressed)?;

            let size_to_read =
                remaining_size.min((compression_block_size - offset_in_block) as usize);

            let buf_offset = buf.len() - remaining_size;
            buf[buf_offset..buf_offset + size_to_read]
                .copy_from_slice(&decompressed[offset_in_block as usize..size_to_read]);

            offset_in_block = 0;
            remaining_size -= size_to_read;

            if remaining_size == 0 {
                break;
            }
        }

        Ok(())
    }

    fn create_partition_reader(&self, partition_index: u64) -> Result<R, Error> {
        match partition_index == 0 {
            true => self
                .provider
                .create_reader_for_file(&format!("{}.ucas", self.file_name)),
            false => self
                .provider
                .create_reader_for_file(&format!("{}_s{}.ucas", self.file_name, partition_index)),
        }
    }
}
