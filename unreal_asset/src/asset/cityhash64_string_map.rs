//! Asset string map hashed with the cityhash64 algorithm

use crate::{containers::indexed_map::IndexedMap, crc, error::Error};

/// Asset string map hashed with the cityhash64 algorithm
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Cityhash64StringMap {
    /// Name map
    map: IndexedMap<u64, String>,
}

impl Cityhash64StringMap {
    /// Create a new `Cityhash64StringMap` instance
    pub fn new() -> Self {
        Cityhash64StringMap {
            map: IndexedMap::new(),
        }
    }

    /// Add a new string
    pub fn add_entry(&mut self, entry: &str) -> Result<(), Error> {
        let hash = crc::generate_import_hash_from_object_path(entry);

        if let Some(existing_entry) = self.map.get_by_key(&hash) {
            if crc::to_lower_string(existing_entry) == crc::to_lower_string(entry) {
                return Ok(());
            }

            return Err(Error::cityhash64_collision(hash, entry.to_string()));
        }

        self.map.insert(hash, entry.to_string());

        Ok(())
    }

    /// Get an entry by hash
    pub fn get_entry(&self, hash: &u64) -> Option<String> {
        self.map.get_by_key(hash).cloned()
    }
}
