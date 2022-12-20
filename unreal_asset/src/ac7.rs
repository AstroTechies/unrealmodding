use crate::UE4_ASSET_MAGIC;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AC7XorKey {
    name_key: i32,
    offset: u32,
    pk1: u32,
    pk2: u32,
}

const AC7_KEY: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/AC7Key.bin"));

impl AC7XorKey {
    pub fn new(name: &str) -> Self {
        let name_key = Self::calc_name_key(name);
        let offset = 4;
        let (pk1, pk2) = Self::calc_pkey_from_nkey(name_key as u32, offset);

        Self {
            name_key,
            offset,
            pk1,
            pk2,
        }
    }

    fn xor_byte(&mut self, byte: u8) -> u8 {
        let byte = byte ^ AC7_KEY[(self.pk1 * 1024 + self.pk2) as usize];
        let byte = byte as u32 ^ 0x77;
        self.pk1 += 1;
        self.pk2 += 1;

        if self.pk1 >= 217 {
            self.pk1 = 0;
        }

        if self.pk2 >= 1024 {
            self.pk2 = 0;
        }

        byte as u8
    }

    fn calc_name_key(name: &str) -> i32 {
        let name = name.to_uppercase();

        let mut num = 0i32;

        for orig_byte in name.as_bytes() {
            let mut num2 = *orig_byte as i32;
            num ^= num2;
            num2 = num.overflowing_mul(8).0;
            num2 ^= num;
            let num3 = num.overflowing_add(num).0;
            num2 = !num2;
            num2 = (num2 >> 7) & 1;
            num = num2 | num3;
        }

        num as i32
    }

    fn calc_pkey_from_nkey(nkey: u32, data_offset: u32) -> (u32, u32) {
        let mut num = nkey as u128 * 7;
        let big_int = 5440514381186227205u128;
        num += data_offset as u128;
        let big_int_2 = big_int * num as u128;

        let mut num_2 = big_int_2 >> 70;
        let mut num_3 = num_2 >> 63;
        num_2 += num_3;
        num_3 = num_2 * 217;
        num -= num_3;

        let pk1 = (num & 0xffffffffu128) as u32;

        let mut num_4 = nkey as u128 * 11;
        num_4 += data_offset as u128;
        num_2 = 0;
        num_2 &= 0x3ff;
        num_4 += num_2;
        num_4 &= 0x3ff;

        let num_5 = num_4 - num_2;
        let pk2 = (num_5 & 0xffffffffu128) as u32;

        (pk1, pk2)
    }
}

pub fn decrypt(uasset: &[u8], uexp: &[u8], mut key: AC7XorKey) -> (Vec<u8>, Vec<u8>) {
    (
        decrypt_uasset(uasset, &mut key),
        decrypt_uexp(uexp, &mut key),
    )
}

pub fn decrypt_uasset(uasset: &[u8], key: &mut AC7XorKey) -> Vec<u8> {
    let mut decrypted = vec![0u8; uasset.len()];
    decrypted[..4].copy_from_slice(&u32::to_be_bytes(UE4_ASSET_MAGIC)); // todo: replace with constant

    for i in 4..uasset.len() {
        decrypted[i] = key.xor_byte(uasset[i]);
    }

    decrypted
}

pub fn decrypt_uexp(uexp: &[u8], key: &mut AC7XorKey) -> Vec<u8> {
    let mut decrypted = vec![0u8; uexp.len()];

    for i in 0..uexp.len() {
        decrypted[i] = key.xor_byte(uexp[i]);
    }

    decrypted
}

pub fn encrypt(uasset: &[u8], uexp: &[u8], mut key: AC7XorKey) -> (Vec<u8>, Vec<u8>) {
    (
        encrypt_uasset(uasset, &mut key),
        encrypt_uexp(uexp, &mut key),
    )
}

const AC7_ASSET_MAGIC: u32 = 0x37454341;

pub fn encrypt_uasset(uasset: &[u8], key: &mut AC7XorKey) -> Vec<u8> {
    let mut encrypted = vec![0u8; uasset.len()];
    encrypted[..4].copy_from_slice(&u32::to_le_bytes(AC7_ASSET_MAGIC));

    for i in 4..uasset.len() {
        encrypted[i] = key.xor_byte(uasset[i]);
    }

    encrypted
}

pub fn encrypt_uexp(uexp: &[u8], key: &mut AC7XorKey) -> Vec<u8> {
    let mut encrypted = vec![0u8; uexp.len()];

    for i in 0..uexp.len() {
        encrypted[i] = key.xor_byte(uexp[i]);
    }

    encrypted
}
