//! Used to represent the version of a pak file

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Enum representing all versions of the pak file format
#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum PakVersion {
    /// Initial version
    PakFileVersionInitial = 1,
    /// Version in which timestamps were removed
    PakFileVersionNoTimestamps = 2,
    /// First version to support compression and encryption
    PakFileVersionCompressionEncryption = 3,
    /// First version to support index encryption
    PakFileVersionIndexEncryption = 4,
    /// Version in which compression chunk offsets were made relative to the header of the file
    PakFileVersionRelativeChunkOffsets = 5,
    /// First version to support delete records
    PakFileVersionDeleteRecords = 6,
    /// First version to include GUID of the used encryption key
    PakFileVersionEncryptionKeyGuid = 7,
    /// First version to list the names of used compression alghorithms
    PakFileVersionFnameBasedCompressionMethod = 8,
    /// Only version which had the frozen index byte
    PakFileVersionFrozenIndex = 9,
    /// Version which reworked how the index is structured
    PakFileVersionPathHashIndex = 10,
    /// Bug Fix version
    PakFileVersionFnv64BugFix = 11,

    /// Last Version of the format
    PakFileVersionLast,
    /// Invalid version
    PakFileVersionInvalid,
}
