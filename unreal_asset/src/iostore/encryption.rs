//! Encryption helpers

use aes::{
    cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, BlockSizeUser},
    Aes256,
};

/// Aes256 block alignment
pub const ENCRYPTION_ALIGN: u64 = 16;

/// Aes256 encryption key
pub type EncryptionKey = [u8; 32];

/// Decrypt data that is aligned to aes256 block size
pub fn decrypt(aes: &Aes256, data: &mut [u8]) {
    data.chunks_mut(Aes256::block_size())
        .map(GenericArray::from_mut_slice)
        .for_each(|e| aes.decrypt_block(e));
}

/// Encrypt datat that is aligned to aes256 block size
pub fn encrypt(aes: &Aes256, data: &mut [u8]) {
    data.chunks_mut(Aes256::block_size())
        .map(GenericArray::from_mut_slice)
        .for_each(|e| aes.encrypt_block(e));
}
