use std::{fmt::Debug, hash::Hash};

use super::IndexedMap;

pub enum Entry<'map, K: 'map, V: 'map>
where
    K: Eq + Hash,
{
    Vacant(VacantEntry<'map, K, V>),
    Occupied(OccupiedEntry<'map, K, V>),
}

impl<'map, K, V> Entry<'map, K, V>
where
    K: Eq + Hash,
{
    pub fn or_insert(self, default: V) -> &'map mut V {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'map mut V {
        match self {
            Entry::Vacant(entry) => entry.insert(default()),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'map mut V {
        match self {
            Entry::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    pub fn key(&self) -> &K {
        match self {
            Entry::Vacant(entry) => entry.key(),
            Entry::Occupied(entry) => entry.key(),
        }
    }
}

impl<K, V> Debug for Entry<'_, K, V>
where
    K: Eq + Hash + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vacant(arg0) => f.debug_tuple("Vacant").field(arg0).finish(),
            Self::Occupied(arg0) => f.debug_tuple("Occupied").field(arg0).finish(),
        }
    }
}

pub struct VacantEntry<'map, K, V>
where
    K: Eq + Hash,
{
    pub(super) key: K,
    pub(super) map: &'map mut IndexedMap<K, V>,
}

/// NOTE: Insertion index is calculated when insert function is called,
/// not when the entry is created.
impl<'map, K, V> VacantEntry<'map, K, V>
where
    K: Eq + Hash,
{
    /// Gets a reference to the key that would be used when inserting a value
    /// through the VacantEntry.
    #[must_use]
    #[inline]
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take ownership of the key.
    #[must_use]
    #[inline]
    pub fn into_key(self) -> K {
        self.key
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    /// NOTE: Insertion index is calculated upon calling this function
    /// Not upon creating the entry
    pub fn insert(self, value: V) -> &'map mut V {
        &mut self.map.internal_insert(self.key, value).value
    }
}

impl<K, V> Debug for VacantEntry<'_, K, V>
where
    K: Eq + Hash + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VacantEntry")
            .field("key", &self.key)
            .finish()
    }
}

pub struct OccupiedEntry<'map, K, V>
where
    K: Eq + Hash,
{
    pub(super) map: &'map mut IndexedMap<K, V>,
    pub(super) store_place: usize,
}

impl<'map, K, V> OccupiedEntry<'map, K, V>
where
    K: Eq + Hash,
{
    #[must_use]
    pub fn key(&self) -> &K {
        self.map.store[self.store_place].key_map_index.0.as_ref()
    }

    pub fn remove_entry(self) -> (usize, K, V) {
        self.map.remove_by_store_place(self.store_place).unwrap()
    }

    #[must_use]
    pub fn get(&self) -> &V {
        &self.map.store[self.store_place].value
    }

    #[must_use]
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.map.store[self.store_place].value
    }

    #[must_use]
    pub fn into_mut(self) -> &'map mut V {
        &mut self.map.store[self.store_place].value
    }

    pub fn remove(self) -> V {
        self.remove_entry().2
    }
}

impl<K, V> Debug for OccupiedEntry<'_, K, V>
where
    K: Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("store_index", &self.store_place)
            .finish()
    }
}
