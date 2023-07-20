//! AC7 Encryption

use std::io::{Read, Seek, Write};

/// A reader which aligns bytes for AC7-encrypted assets
pub struct AC7Reader<C: Read + Seek> {
    key: AC7XorKey,
    inner: C,
}

impl<C: Read + Seek> AC7Reader<C> {
    /// Creates a new AC7Reader for an asset with the specified key
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use unreal_asset::{Asset, EngineVersion, ac7::*};
    ///
    /// let key = AC7XorKey::new("ex02_IGC_03_Subtitle");
    /// let mut asset = Asset::new(
    ///     AC7Reader::new(key, File::open("ex02_IGC_03_Subtitle.uasset").unwrap()),
    ///     Some(AC7Reader::new(key, File::open("ex02_IGC_03_Subtitle.uexp").unwrap()))
    ///     EngineVersion::VER_UE4_18,
    /// )?;
    /// ```
    pub fn new(key: AC7XorKey, inner: C) -> Self {
        Self { key, inner }
    }
    /// Consumes this reader, returning the underlying value.
    pub fn into_inner(self) -> C {
        self.inner
    }
}

impl<C: Read + Seek> Read for AC7Reader<C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf);
        for byte in buf.iter_mut() {
            *byte = self.key.xor_byte(*byte)
        }
        read
    }
}

impl<C: Read + Seek> Seek for AC7Reader<C> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

/// A writer which aligns bytes for AC7-encrypted assets
pub struct AC7Writer<C: Seek + Write> {
    key: AC7XorKey,
    inner: C,
}

impl<C: Seek + Write> AC7Writer<C> {
    /// Creates a new AC7Reader for an asset with the specified key
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use unreal_asset::{Asset, EngineVersion, ac7::*};
    ///
    /// let key = AC7XorKey::new("ex02_IGC_03_Subtitle");
    /// let mut asset = Asset::new(
    ///     AC7Reader::new(key, File::open("ex02_IGC_03_Subtitle.uasset").unwrap()),
    ///     Some(AC7Reader::new(key, File::open("ex02_IGC_03_Subtitle.uexp").unwrap()))
    ///     EngineVersion::VER_UE4_18,
    /// )?;
    /// asset.write_data(
    ///      AC7Writer::new(key, File::create("ex02_IGC_03_Subtitle.uasset")),
    ///      Some(AC7Writer::new(key, File::create("ex02_IGC_03_Subtitle.uexp"))
    /// ))?;
    /// ```
    pub fn new(key: AC7XorKey, inner: C) -> Self {
        Self { key, inner }
    }
    /// Consumes this writer, returning the underlying value.
    pub fn into_inner(self) -> C {
        self.inner
    }
}

impl<C: Seek + Write> Seek for AC7Writer<C> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<C: Seek + Write> Write for AC7Writer<C> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;
        for byte in buf.iter().map(|byte| self.key.xor_byte(*byte)) {
            written += self.inner.write(&[byte])?;
        }
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// AC7 Encryption xor key
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AC7XorKey {
    name_key: i32,
    offset: u32,
    pk1: u32,
    pk2: u32,
}

const AC7_KEY: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/AC7Key.bin"));

impl AC7XorKey {
    /// Creates a new AC7XorKey for an asset with the specified name
    /// Note: name should be without extension
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

    /// Process a single byte with this key
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

    /// Calculate a name key for a given name
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

        num
    }

    /// Calculate private key from name key
    fn calc_pkey_from_nkey(nkey: u32, data_offset: u32) -> (u32, u32) {
        let mut num = nkey as u128 * 7;
        let big_int = 5440514381186227205u128;
        num += data_offset as u128;
        let big_int_2 = big_int * num;

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
