//! Various iostore flags

use bitflags::bitflags;

bitflags! {
    /// IoStore container flags
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct EIoContainerFlags : u8 {
        /// None
        const NONE = 0x0;
        /// Compressed
        const COMPRESSED = (1 << 0);
        /// Encrypted
        const ENCRYPTED = (1 << 1);
        /// Signed
        const SIGNED = (1 << 2);
        /// Indexed
        const INDEXED = (1 << 3);
    }

    /// IoStore .utoc entry metadata flags
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct IoStoreTocEntryMetaFlags : u8 {
        /// None
        const NONE = 0x0;
        /// Compressed
        const COMPRESSED = (1 << 0);
        /// Memory mapped
        const MEMORY_MAPPED = (1 << 1);
    }

    /// IoStore export filter flags
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct EExportFilterFlags : u8 {
        /// None
        const NONE = 0x0;
        /// Not for client
        const NOT_FOR_CLIENT = 0x1;
        /// Not for server
        const NOT_FOR_SERVER = 0x2;
    }
}
