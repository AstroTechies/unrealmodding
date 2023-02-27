//! Used to represent the version of a pak file

/// Enum representing all versions of the pak file format
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PakVersion {
    /// Initial version
    Initial,
    /// Version in which timestamps were removed
    NoTimestamps,
    /// First version to support compression and encryption
    CompressionEncryption,
    /// First version to support index encryption
    IndexEncryption,
    /// Version in which compression chunk offsets were made relative to the header of the file
    RelativeChunkOffsets,
    /// First version to support delete records
    DeleteRecords,
    /// First version to include GUID of the used encryption key
    EncryptionKeyGuid,
    /// Weird version used by just UE 4.22. Also sometimes called pak v8a.
    FnameBasedCompressionMethodInitial,
    /// First version to list the names of used compression alghorithms. Also sometimes called pak v8b.
    FnameBasedCompressionMethod,
    /// Only version which had the frozen index byte
    FrozenIndex,
    /// Version which reworked how the index is structured
    PathHashIndex,
    /// Bug Fix version
    Fnv64BugFix,

    /// Invalid version
    Invalid,
}

impl PakVersion {
    /// Create version from a u32.
    pub fn from_num(version: u32) -> Self {
        match version {
            1 => Self::Initial,
            2 => Self::NoTimestamps,
            3 => Self::CompressionEncryption,
            4 => Self::IndexEncryption,
            5 => Self::RelativeChunkOffsets,
            6 => Self::DeleteRecords,
            7 => Self::EncryptionKeyGuid,
            8 => Self::FnameBasedCompressionMethod,
            9 => Self::FrozenIndex,
            10 => Self::PathHashIndex,
            11 => Self::Fnv64BugFix,
            _ => Self::Invalid,
        }
    }

    /// Convert version to a u32.
    pub fn to_num(&self) -> u32 {
        match self {
            Self::Initial => 1,
            Self::NoTimestamps => 2,
            Self::CompressionEncryption => 3,
            Self::IndexEncryption => 4,
            Self::RelativeChunkOffsets => 5,
            Self::DeleteRecords => 6,
            Self::EncryptionKeyGuid => 7,
            Self::FnameBasedCompressionMethodInitial => 8,
            Self::FnameBasedCompressionMethod => 8,
            Self::FrozenIndex => 9,
            Self::PathHashIndex => 10,
            Self::Fnv64BugFix => 11,
            Self::Invalid => panic!("Attempted to write invalid pak version as byte!"),
        }
    }

    // how to deal with the stupid 4.22 version
    pub(crate) fn set_subversion(&mut self) {
        if matches!(self, Self::FnameBasedCompressionMethod) {
            *self = Self::FnameBasedCompressionMethodInitial;
        }
    }
}
