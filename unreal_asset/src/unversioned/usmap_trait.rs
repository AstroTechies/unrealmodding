//! Usmap data trait

use std::io;

/// .usmap Data trait
pub trait UsmapTrait {
    /// Get .usmap asset position
    fn position(&mut self) -> u64;

    /// Get .usmap asset length
    fn stream_length(&mut self) -> io::Result<u64>;
}
