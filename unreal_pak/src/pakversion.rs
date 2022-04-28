use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum PakVersion {
    PakFileVersionInitial = 1,
    PakFileVersionNoTimestamps = 2,
    PakFileVersionCompressionEncryption = 3,
    PakFileVersionIndexEncryption = 4,
    PakFileVersionRelativeChunkOffsets = 5,
    PakFileVersionDeleteRecords = 6,
    PakFileVersionEncryptionKeyGuid = 7,
    PakFileVersionFnameBasedCompressionMethod = 8,
    PakFileVersionFrozenIndex = 9,
    PakFileVersionPathHashIndex = 10,
    PakFileVersionFnv64bugFix = 11,

    PakFileVersionLast,
    PakFileVersionInvalid,
}
