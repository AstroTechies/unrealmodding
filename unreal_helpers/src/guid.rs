//! [`Guid`] type

use std::{
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
};

/// Stores a 128-bit guid (globally unique identifier)
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Guid(pub [u8; 16]);

impl Guid {
    /// Create new instance of Guid struct from a [0u8; 16] byte array
    #[inline]
    pub fn new(guid: [u8; 16]) -> Self {
        Guid(guid)
    }

    /// Returns true if the guid is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }
}

impl From<[u8; 16]> for Guid {
    /// Create new instance of Guid struct from an array.
    #[inline]
    fn from(value: [u8; 16]) -> Self {
        Guid(value)
    }
}

impl From<Guid> for [u8; 16] {
    /// Convert Guid struct into an array.
    #[inline]
    fn from(value: Guid) -> Self {
        value.0
    }
}

impl From<(u32, u32, u32, u32)> for Guid {
    /// Create new instance of Guid struct from 4 u32 values.
    #[inline]
    fn from(value: (u32, u32, u32, u32)) -> Self {
        let (a, b, c, d) = value;
        Guid([
            (a & 0xff) as u8,
            ((a >> 8) & 0xff) as u8,
            ((a >> 16) & 0xff) as u8,
            ((a >> 24) & 0xff) as u8,
            (b & 0xff) as u8,
            ((b >> 8) & 0xff) as u8,
            ((b >> 16) & 0xff) as u8,
            ((b >> 24) & 0xff) as u8,
            (c & 0xff) as u8,
            ((c >> 8) & 0xff) as u8,
            ((c >> 16) & 0xff) as u8,
            ((c >> 24) & 0xff) as u8,
            (d & 0xff) as u8,
            ((d >> 8) & 0xff) as u8,
            ((d >> 16) & 0xff) as u8,
            ((d >> 24) & 0xff) as u8,
        ])
    }
}

impl From<Guid> for (u32, u32, u32, u32) {
    /// Convert Guid struct into 4 u32 values.
    #[inline]
    fn from(guid: Guid) -> Self {
        let a = guid.0[0] as u32
            | ((guid.0[1] as u32) << 8)
            | ((guid.0[2] as u32) << 16)
            | ((guid.0[3] as u32) << 24);
        let b = guid.0[4] as u32
            | ((guid.0[5] as u32) << 8)
            | ((guid.0[6] as u32) << 16)
            | ((guid.0[7] as u32) << 24);
        let c = guid.0[8] as u32
            | ((guid.0[9] as u32) << 8)
            | ((guid.0[10] as u32) << 16)
            | ((guid.0[11] as u32) << 24);
        let d = guid.0[12] as u32
            | ((guid.0[13] as u32) << 8)
            | ((guid.0[14] as u32) << 16)
            | ((guid.0[15] as u32) << 24);

        (a, b, c, d)
    }
}

impl From<u128> for Guid {
    #[inline]
    fn from(value: u128) -> Self {
        Guid([
            (value & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            ((value >> (8 * 2)) & 0xff) as u8,
            ((value >> (8 * 3)) & 0xff) as u8,
            ((value >> (8 * 4)) & 0xff) as u8,
            ((value >> (8 * 5)) & 0xff) as u8,
            ((value >> (8 * 6)) & 0xff) as u8,
            ((value >> (8 * 7)) & 0xff) as u8,
            ((value >> (8 * 8)) & 0xff) as u8,
            ((value >> (8 * 9)) & 0xff) as u8,
            ((value >> (8 * 10)) & 0xff) as u8,
            ((value >> (8 * 11)) & 0xff) as u8,
            ((value >> (8 * 12)) & 0xff) as u8,
            ((value >> (8 * 13)) & 0xff) as u8,
            ((value >> (8 * 14)) & 0xff) as u8,
            ((value >> (8 * 15)) & 0xff) as u8,
        ])
    }
}

impl From<Guid> for u128 {
    #[inline]
    fn from(value: Guid) -> Self {
        (value.0[0] as u128)
            | ((value.0[1] as u128) << 8)
            | ((value.0[2] as u128) << (8 * 2))
            | ((value.0[3] as u128) << (8 * 3))
            | ((value.0[4] as u128) << (8 * 4))
            | ((value.0[5] as u128) << (8 * 5))
            | ((value.0[6] as u128) << (8 * 6))
            | ((value.0[7] as u128) << (8 * 7))
            | ((value.0[8] as u128) << (8 * 8))
            | ((value.0[9] as u128) << (8 * 9))
            | ((value.0[10] as u128) << (8 * 10))
            | ((value.0[11] as u128) << (8 * 11))
            | ((value.0[12] as u128) << (8 * 12))
            | ((value.0[13] as u128) << (8 * 13))
            | ((value.0[14] as u128) << (8 * 14))
            | ((value.0[15] as u128) << (8 * 15))
    }
}

impl Debug for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guid = self.to_string();
        write!(f, "Guid({})", &guid)
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            write!(f, "0")?;
            return Ok(());
        }

        write!(f, "{:02X}", self.0[0])?;
        write!(f, "{:02X}", self.0[1])?;
        write!(f, "{:02X}", self.0[2])?;
        write!(f, "{:02X}", self.0[3])?;

        write!(f, "-")?;

        write!(f, "{:02X}", self.0[4])?;
        write!(f, "{:02X}", self.0[5])?;

        write!(f, "-")?;

        write!(f, "{:02X}", self.0[6])?;
        write!(f, "{:02X}", self.0[7])?;

        write!(f, "-")?;

        write!(f, "{:02X}", self.0[8])?;
        write!(f, "{:02X}", self.0[9])?;

        write!(f, "-")?;

        write!(f, "{:02X}", self.0[10])?;
        write!(f, "{:02X}", self.0[11])?;
        write!(f, "{:02X}", self.0[12])?;
        write!(f, "{:02X}", self.0[13])?;
        write!(f, "{:02X}", self.0[14])?;
        write!(f, "{:02X}", self.0[15])?;
        Ok(())
    }
}

/// An error ocurred while parsing a Guid
#[derive(Debug)]
pub struct ParseGuidError;

impl Display for ParseGuidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid GUID syntax")
    }
}

impl Error for ParseGuidError {}

impl FromStr for Guid {
    type Err = ParseGuidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cleaned = s.replace('-', "");
        let cleaned = cleaned.trim();
        let cleaned = cleaned.strip_prefix('{').unwrap_or(cleaned);
        let cleaned = cleaned.strip_suffix('}').unwrap_or(cleaned);

        if cleaned.len() == 1 && cleaned == "0" {
            return Ok(Guid::new([0u8; 16]));
        }

        if cleaned.len() != 32 {
            Err(ParseGuidError)?;
        }
        let mut guid = Guid(Default::default());
        for i in 0..16 {
            guid.0[i] =
                u8::from_str_radix(&cleaned[i * 2..i * 2 + 2], 16).map_err(|_| ParseGuidError)?;
        }
        Ok(guid)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Guid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Guid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Guid::from_str(&s).map_err(serde::de::Error::custom)
    }
}
