use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::Debug,
    hash::{self, Hash},
    iter::FusedIterator,
    rc::Rc,
};

use self::entry::{Entry, OccupiedEntry, VacantEntry};

pub mod entry;

// todo: more docs

/// Used for storing a key reference inside IndexedMap
#[derive(PartialEq, Eq, Hash)]
pub struct KeyItem<K: Eq + Hash>(Rc<K>);

impl<K> Clone for KeyItem<K>
where
    K: Eq + Hash,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K> Debug for KeyItem<K>
where
    K: Eq + Hash + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("KeyItem").field(&self.0).finish()
    }
}

impl<K> Borrow<K> for KeyItem<K>
where
    K: Eq + Hash,
{
    fn borrow(&self) -> &K {
        self.0.as_ref()
    }
}

impl<K> Borrow<str> for KeyItem<K>
where
    K: Eq + Hash + Borrow<str>,
{
    fn borrow(&self) -> &str {
        self.0.as_ref().borrow()
    }
}

/// Used for storing values inside IndexedMap
/// Contains references to the value key and index in the internal index map
pub struct IndexedValue<K, V>
where
    K: Eq + Hash,
{
    value: V,
    /// A reference to an index inside key_map
    key_map_index: KeyItem<K>,
    /// An index inside the index_map
    index_map_index: usize,
    /// An index inside the index_iter_map, index_iter_map is used for faster iteration when iterating by index.
    index_iter_map_index: usize,
}

impl<K, V> Clone for IndexedValue<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            key_map_index: self.key_map_index.clone(),
            index_map_index: self.index_map_index.clone(),
            index_iter_map_index: self.index_iter_map_index.clone(),
        }
    }
}

impl<K, V> PartialEq for IndexedValue<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.key_map_index == other.key_map_index
            && self.index_map_index == other.index_map_index
            && self.index_iter_map_index == other.index_iter_map_index
    }
}

impl<K, V> Eq for IndexedValue<K, V>
where
    K: Eq + Hash,
    V: Eq,
{
}

impl<K, V> Debug for IndexedValue<K, V>
where
    K: Eq + Hash + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexedValue")
            .field("value", &self.value)
            .field("key_map_index", &self.key_map_index)
            .field("index_map_index", &self.index_map_index)
            .field("index_iter_map_index", &self.index_iter_map_index)
            .finish()
    }
}

/// A hashmap that stores insertion index and allows retrieval
/// by key or insertion index.
///
/// Insertion time is O(1)
/// Deletion time is O(n) worst-case
pub struct IndexedMap<K, V>
where
    K: Eq + Hash,
{
    /// Stores all the values
    pub store: slab::Slab<IndexedValue<K, V>>,
    /// Key reference -> store reference
    pub key_map: rustc_hash::FxHashMap<KeyItem<K>, usize>,
    /// Insertion index -> store reference
    pub index_map: BTreeMap<usize, usize>,
    /// Faster iteration over store, because iterating over a vector is much faster
    /// than iterating over a [`std::collections::btree_map::Iter`]
    pub index_iter_map: Vec<usize>,
}

/// Iterates in the insertion index order on an [`IndexedMap`]
pub struct IndexedMapIndexIterator<'map, K, V>
where
    K: Eq + Hash,
{
    store: &'map slab::Slab<IndexedValue<K, V>>,
    index_iter: std::slice::Iter<'map, usize>,
}

impl<'map, K, V> Iterator for IndexedMapIndexIterator<'map, K, V>
where
    K: Eq + Hash,
{
    type Item = (usize, &'map K, &'map V);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index_iter.next()?;
        let value = &self.store[*index];
        Some((*index, value.key_map_index.0.as_ref(), &value.value))
    }
}

impl<'map, K, V> DoubleEndedIterator for IndexedMapIndexIterator<'map, K, V>
where
    K: Eq + Hash,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.index_iter.next_back()?;
        let value = &self.store[*index];
        Some((*index, value.key_map_index.0.as_ref(), &value.value))
    }
}

impl<K, V> ExactSizeIterator for IndexedMapIndexIterator<'_, K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}

impl<K, V> FusedIterator for IndexedMapIndexIterator<'_, K, V> where K: Eq + Hash {}

impl<K, V> Clone for IndexedMapIndexIterator<'_, K, V>
where
    K: Eq + Hash,
{
    fn clone(&self) -> Self {
        IndexedMapIndexIterator {
            store: self.store,
            index_iter: self.index_iter.clone(),
        }
    }
}

/// Iterates in the internal key map iterator order over an [`IndexedMap`]
pub struct IndexedMapKeyIterator<'map, K, V>
where
    K: Eq + Hash,
{
    store: &'map slab::Slab<IndexedValue<K, V>>,
    key_map_iter: std::collections::hash_map::Iter<'map, KeyItem<K>, usize>,
}

impl<'map, K, V> Iterator for IndexedMapKeyIterator<'map, K, V>
where
    K: Eq + Hash,
{
    type Item = (usize, &'map K, &'map V);

    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = self.key_map_iter.next()?;
        let value = &self.store[*value];
        Some((
            value.index_map_index,
            value.key_map_index.0.as_ref(),
            &value.value,
        ))
    }
}

impl<K, V> ExactSizeIterator for IndexedMapKeyIterator<'_, K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn len(&self) -> usize {
        self.key_map_iter.len()
    }
}

impl<K, V> FusedIterator for IndexedMapKeyIterator<'_, K, V> where K: Eq + Hash {}

impl<K, V> Clone for IndexedMapKeyIterator<'_, K, V>
where
    K: Eq + Hash,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store,
            key_map_iter: self.key_map_iter.clone(),
        }
    }
}

/// Iterates over [`IndexedMap`] values
#[derive(Clone)]
pub struct Values<'map, K, V>
where
    K: Eq + Hash,
{
    inner: IndexedMapIndexIterator<'map, K, V>,
}

impl<'map, K, V> Iterator for Values<'map, K, V>
where
    K: Eq + Hash,
{
    type Item = &'map V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, _, v)| v)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'map, K, V> DoubleEndedIterator for Values<'map, K, V>
where
    K: Eq + Hash,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, _, v)| v)
    }
}

impl<K, V> ExactSizeIterator for Values<'_, K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Values<'_, K, V> where K: Eq + Hash {}

/// Iterates over [`IndexedMap`] keys
#[derive(Clone)]
pub struct Keys<'map, K, V>
where
    K: Eq + Hash,
{
    inner: IndexedMapIndexIterator<'map, K, V>,
}

impl<'map, K, V> Iterator for Keys<'map, K, V>
where
    K: Eq + Hash,
{
    type Item = &'map K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, k, _)| k)
    }
}

impl<'map, K, V> DoubleEndedIterator for Keys<'map, K, V>
where
    K: Eq + Hash,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, k, _)| k)
    }
}

impl<K, V> ExactSizeIterator for Keys<'_, K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> FusedIterator for Keys<'_, K, V> where K: Eq + Hash {}

impl<'map, K, V> IndexedMap<K, V>
where
    K: Eq + Hash,
{
    /// Create a new instance of [`IndexedMap`]
    pub fn new() -> Self {
        IndexedMap {
            store: slab::Slab::new(),
            key_map: rustc_hash::FxHashMap::default(),
            index_map: BTreeMap::new(),
            index_iter_map: Vec::new(),
        }
    }

    /// Create a new instance of [`IndexedMap`] with a preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        IndexedMap {
            store: slab::Slab::with_capacity(capacity),
            key_map: rustc_hash::FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            index_map: BTreeMap::new(),
            index_iter_map: Vec::new(),
        }
    }

    /// Inserts a value at a given key, if the key already exists
    /// replaces existing value.
    ///
    /// Returns a mutable reference to the inserted value
    fn internal_insert(&'map mut self, key: K, value: V) -> &'map mut IndexedValue<K, V> {
        if let Some(storage_place) = self.key_map.get(&key) {
            self.store[*storage_place].value = value;
            return &mut self.store[*storage_place];
        }

        let key_rc = KeyItem(Rc::new(key));
        let indexed_value = IndexedValue {
            value,
            key_map_index: key_rc.clone(),
            index_map_index: self.key_map.len(),
            index_iter_map_index: self.index_iter_map.len(),
        };
        let store_place = self.store.insert(indexed_value);

        self.key_map.insert(key_rc.clone(), store_place);
        self.index_map.insert(self.key_map.len() - 1, store_place);
        self.index_iter_map.push(store_place);
        return &mut self.store[store_place];
    }

    /// Inserts a value at a given key, if the key already exists
    /// replaces existing value.
    pub fn insert(&mut self, key: K, value: V) {
        self.internal_insert(key, value);
    }

    /// Creates an [`Entry`] for a given key
    pub fn entry(&'map mut self, key: K) -> Entry<'map, K, V> {
        if let Some(store_place) = self.key_map.get(&key) {
            let place = *store_place;
            return Entry::Occupied(OccupiedEntry {
                map: self,
                store_place: place,
            });
        }

        return Entry::Vacant(VacantEntry { key, map: self });
    }

    /// Gets a reference for a value associated with the given key
    ///
    /// If there is no value associated with a given key, `None` is returned
    pub fn get_by_key<Q>(&self, key: &Q) -> Option<&V>
    where
        KeyItem<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(store_place) = self.key_map.get(key) {
            return self.store.get(*store_place).map(|e| &e.value);
        }

        None
    }

    /// Gets a mutable reference for a value associated with the given key
    ///
    /// If there is no value associated with a given key, `None` is returned.
    pub fn get_by_key_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        KeyItem<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(store_place) = self.key_map.get(key) {
            return self.store.get_mut(*store_place).map(|e| &mut e.value);
        }

        None
    }

    /// Gets a reference for a value associated with the given insertion index
    ///
    /// If there is no value associated with a given insertion index, `None` is returned.
    pub fn get_by_index(&self, index: usize) -> Option<&V> {
        if let Some(store_place) = self.index_map.get(&index) {
            return self.store.get(*store_place).map(|e| &e.value);
        }

        None
    }

    /// Gets a mutable reference for a value associated with the given insertion index
    ///
    /// If there is no value associated with a given insertion index, 'None' is returned.
    pub fn get_by_index_mut(&mut self, index: usize) -> Option<&mut V> {
        if let Some(store_place) = self.index_map.get(&index) {
            return self.store.get_mut(*store_place).map(|e| &mut e.value);
        }

        None
    }

    /// Returns the number of elements in a map
    pub fn len(&self) -> usize {
        self.index_map.len()
    }

    /// Returns an iterator over the values of the map.
    pub fn values(&'map self) -> Values<'map, K, V> {
        Values { inner: self.iter() }
    }

    /// Returns an iterator over the keys of the map.
    pub fn keys(&'map self) -> Keys<'map, K, V> {
        Keys { inner: self.iter() }
    }

    /// Removes a value by store place
    /// If the value existed in the store and the key did not have any
    /// references to it, the Some((index, key, value)) pair is returned,
    /// otherwise, None is returned.
    fn remove_by_store_place(&mut self, store_place: usize) -> Option<(usize, K, V)> {
        if !self.store.contains(store_place) {
            return None;
        };

        let value = self.store.remove(store_place);

        self.index_iter_map.remove(value.index_iter_map_index);
        self.index_map.remove(&value.index_map_index);
        self.key_map.remove(value.key_map_index.0.as_ref());

        Some((
            value.index_map_index,
            Rc::<K>::try_unwrap(value.key_map_index.0).ok()?,
            value.value,
        ))
    }

    /// Removes a value by index
    /// If the value existed in a map and the key did not have any
    /// references to it, the Some((index, key, value)) pair is returned,
    /// otherwise, None is returned.
    pub fn remove_by_index(&mut self, index: usize) -> Option<(usize, K, V)> {
        let Some(store_place) = self.index_map.get(&index) else {
            return None;
        };

        self.remove_by_store_place(*store_place)
    }

    /// Removes a value by key
    /// If the value existed in a map and the key did not have any
    /// references to it, the Some((index, key, value)) pair is returned,
    /// otherwise, None is returned.
    pub fn remove_by_key<Q>(&mut self, key: &Q) -> Option<(usize, K, V)>
    where
        KeyItem<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let Some(store_place) = self.key_map.get(key) else {
            return None;
        };

        self.remove_by_store_place(*store_place)
    }

    /// Checks if a given key exists in a map
    pub fn contains_key(&self, key: &K) -> bool {
        self.key_map.contains_key(key)
    }

    /// Checks if a given insertion index exists in a map
    pub fn contains_index(&self, index: usize) -> bool {
        self.index_map.contains_key(&index)
    }

    /// Returns an iterator in the insertion index order
    pub fn iter(&'map self) -> IndexedMapIndexIterator<'map, K, V> {
        IndexedMapIndexIterator {
            store: &self.store,
            index_iter: self.index_iter_map.iter(),
        }
    }

    /// Returns an iterator in the key order
    pub fn iter_key(&'map self) -> IndexedMapKeyIterator<'map, K, V> {
        IndexedMapKeyIterator {
            store: &self.store,
            key_map_iter: self.key_map.iter(),
        }
    }
}

impl<K, V> PartialEq for IndexedMap<K, V>
where
    K: Eq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        self.key_map == other.key_map
            && self.index_map == other.index_map
            && self.index_iter_map == other.index_iter_map
    }
}

impl<K, V> Eq for IndexedMap<K, V> where K: Eq + Hash {}

impl<K, V> Clone for IndexedMap<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            key_map: self.key_map.clone(),
            index_map: self.index_map.clone(),
            index_iter_map: self.index_iter_map.clone(),
        }
    }
}

impl<K, V> Debug for IndexedMap<K, V>
where
    K: Eq + Hash + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexedMap")
            .field("store", &self.store)
            .field("key_map", &self.key_map)
            .field("index_map", &self.index_map)
            .field("index_iter_map", &self.index_iter_map)
            .finish()
    }
}

impl<K, V> Extend<(K, V)> for IndexedMap<K, V>
where
    K: Eq + Hash,
{
    #[inline]
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<'map, K, V> IntoIterator for &'map IndexedMap<K, V>
where
    K: Eq + Hash,
{
    type Item = (usize, &'map K, &'map V);
    type IntoIter = IndexedMapIndexIterator<'map, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IndexedMapIndexIterator {
            store: &self.store,
            index_iter: self.index_iter_map.iter(),
        }
    }
}

impl<K, V> FromIterator<(K, V)> for IndexedMap<K, V>
where
    K: Eq + Hash,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = IndexedMap::new();
        map.extend(iter);
        map
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for IndexedMap<K, V>
where
    K: Eq + Hash,
{
    fn from(value: [(K, V); N]) -> Self {
        Self::from_iter(value)
    }
}
