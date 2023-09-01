//! IoStore FNames

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use unreal_asset_base::{
    crc, enums,
    object_version::ObjectVersion,
    reader::{ArchiveReader, ArchiveWriter},
    types::{PackageIndexTrait, SerializedNameHeader},
    Error,
};

/// IoStore mapped name type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EMappedNameType {
    /// Package-level name table
    Package,
    /// Container-level name table
    Container,
    /// Global name table
    Global,
}

/// IoStore mapped name
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FMappedName {
    /// Name index
    pub index: u32,
    /// Name number
    pub number: u32,
    /// Name type
    pub ty: EMappedNameType,
}

impl FMappedName {
    /// FMappedName index bits
    pub const INDEX_BITS: u32 = 30;
    /// FMappedName index mask
    pub const INDEX_MASK: u32 = (1u32 << Self::INDEX_BITS).overflowing_sub(1).0;
    /// FMappedName type mask
    pub const TYPE_MASK: u32 = !Self::INDEX_MASK;
    /// FMappedName type shift
    pub const TYPE_SHIFT: u32 = Self::INDEX_BITS;

    /// Create a new `FMappedName` instance
    pub fn new(index: u32, number: u32, ty: EMappedNameType) -> Self {
        FMappedName { index, number, ty }
    }

    /// Read `FMappedName` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let index = archive.read_u32::<LE>()?;
        let number = archive.read_u32::<LE>()?;

        let ty = EMappedNameType::try_from(((index & Self::TYPE_MASK) >> Self::TYPE_SHIFT) as u8)?;

        Ok(FMappedName {
            index: index & Self::INDEX_MASK,
            number,
            ty,
        })
    }

    /// Write `FMappedName` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        let index = self.index & Self::INDEX_MASK | (self.ty as u32) << Self::TYPE_SHIFT;

        archive.write_u32::<LE>(index)?;
        archive.write_u32::<LE>(self.number)?;

        Ok(())
    }
}

/// IoStore serialized fname entry
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FNameEntrySerialized {
    /// Name
    pub name: Option<String>,
}

impl FNameEntrySerialized {
    /// Create a new `FNameEntrySerialized` instance
    pub fn new(name: Option<String>) -> Self {
        FNameEntrySerialized { name }
    }

    /// Read `FNameEntrySerialized` from an archive
    pub fn read<R: ArchiveReader<impl PackageIndexTrait>>(archive: &mut R) -> Result<Self, Error> {
        let name = archive.read_fstring()?;

        if archive.get_object_version() >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED {
            let _non_case_preserving_hash = archive.read_u16::<LE>()?;
            let _case_preserving_hash = archive.read_u16::<LE>()?;
        }

        Ok(FNameEntrySerialized { name })
    }

    /// Write `FNameEntrySerialized` to an archive
    pub fn write<W: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        archive: &mut W,
    ) -> Result<(), Error> {
        archive.write_fstring(self.name.as_deref())?;

        if archive.get_object_version() >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED {
            let non_case_preserving_hash = self
                .name
                .as_ref()
                .map(|e| crc::non_case_preserving_hash(e.as_str()))
                .unwrap_or(0);
            let case_preserving_hash = self
                .name
                .as_ref()
                .map(|e| crc::case_preserving_hash(e.as_str()))
                .unwrap_or(0);

            archive.write_u16::<LE>(non_case_preserving_hash)?;
            archive.write_u16::<LE>(case_preserving_hash)?;
        }

        Ok(())
    }

    /// Read an `FNameEntrySerialized` name batch from an archive
    pub fn read_name_batch<R: ArchiveReader<impl PackageIndexTrait>>(
        archive: &mut R,
    ) -> Result<Vec<Self>, Error> {
        let num_strings = archive.read_i32::<LE>()?;
        if num_strings == 0 {
            return Ok(Vec::new());
        }

        let _strings_length = archive.read_u32::<LE>()?;
        let hash_version = archive.read_u64::<LE>()?;

        let _hashes = match hash_version {
            hash if hash == enums::HASH_VERSION_CITYHASH64 => {
                let mut hashes = Vec::with_capacity(num_strings as usize);
                for _ in 0..num_strings {
                    hashes.push(archive.read_u64::<LE>()?); // cityhash64 of crc::to_lower_string
                }
                Ok(hashes)
            }
            _ => Err(Error::unimplemented(format!(
                "Unimplemented name batch algorithm: {}",
                hash_version
            ))),
        }?;

        let headers = archive
            .read_array_with_length(num_strings, |reader| SerializedNameHeader::read(reader))?;

        let mut entries = Vec::with_capacity(num_strings as usize);
        for header in headers {
            entries.push(FNameEntrySerialized::new(
                archive.read_fstring_len_noterm(header.len, header.is_wide)?,
            ));
        }

        Ok(entries)
    }

    /// Read an `FNameEntrySerialized` name batch from an archive using the old method
    pub fn read_name_batch_old<R: ArchiveReader<impl PackageIndexTrait>>(
        archive: &mut R,
        length: usize,
    ) -> Result<Vec<Self>, Error> {
        let mut entries = Vec::with_capacity(length);
        for _ in 0..length {
            let header = SerializedNameHeader::read(archive)?;
            let name = FNameEntrySerialized::new(
                archive.read_fstring_len_noterm(header.len, header.is_wide)?,
            );
            entries.push(name);
        }

        Ok(entries)
    }
}
