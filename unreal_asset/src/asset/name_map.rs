//! Asset name map

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::{
    containers::{
        indexed_map::IndexedMap,
        shared_resource::{CyclicSharedResource, SharedResource, SharedResourceWeakRef},
    },
    types::fname::{FName, EMappedNameType},
};

/// Asset name map
#[derive(Debug, Clone)]
pub struct NameMap {
    /// Name map lookup
    name_map_lookup: IndexedMap<u64, i32>,
    /// Name map index list
    name_map_index_list: Vec<String>,
    /// A reference to self
    self_ref: SharedResourceWeakRef<NameMap>,
}

impl NameMap {
    /// Creates a new `NameMap` instance
    pub fn new() -> SharedResource<NameMap> {
        SharedResource::new_cyclic(|me| NameMap {
            name_map_lookup: IndexedMap::new(),
            name_map_index_list: Vec::new(),
            self_ref: me.clone(),
        })
    }

    /// Creates a new `NameMap` instance from a name batch
    pub fn from_name_batch(name_batch: &[String]) -> SharedResource<Self> {
        let mut name_map = NameMap::new();
        name_map.get_mut().name_map_index_list = Vec::with_capacity(name_batch.len());

        for name in name_batch {
            name_map.get_mut().add_name_reference(name.clone(), false);
        }

        name_map
    }

    /// Search an FName reference
    pub fn search_name_reference(&self, name: &str) -> Option<i32> {
        let mut s = DefaultHasher::new();
        name.hash(&mut s);

        self.name_map_lookup.get_by_key(&s.finish()).copied()
    }

    /// Add an FName reference
    pub fn add_name_reference(&mut self, name: String, force_add_duplicates: bool) -> i32 {
        if !force_add_duplicates {
            let existing = self.search_name_reference(&name);
            if let Some(existing) = existing {
                return existing;
            }
        }

        let mut s = DefaultHasher::new();
        name.hash(&mut s);

        let hash = s.finish();
        self.name_map_index_list.push(name.clone());
        self.name_map_lookup
            .insert(hash, (self.name_map_index_list.len() - 1) as i32);
        (self.name_map_lookup.len() - 1) as i32
    }

    /// Get all FNames
    pub fn get_name_map_index_list(&self) -> &[String] {
        &self.name_map_index_list
    }

    /// Get a name reference by an FName map index
    pub fn get_name_reference(&self, index: i32) -> String {
        if index < 0 {
            return (-index).to_string(); // is this right even?
        }
        if index >= self.name_map_index_list.len() as i32 {
            return index.to_string();
        }
        self.name_map_index_list[index as usize].to_owned()
    }

    /// Get a mutable name reference by an FName map index
    pub fn get_name_reference_mut(&mut self, index: i32) -> &mut String {
        &mut self.name_map_index_list[index as usize]
    }

    /// Create an `FName` for an index in this name map
    pub fn create_fname(&self, index: i32, number: i32) -> FName {
        FName::Backed {
            index,
            number,
            ty: EMappedNameType::Package,
            name_map: self.self_ref.upgrade().unwrap(),
        }
    }

    /// Add an `FName`
    pub fn add_fname(&mut self, slice: &str) -> FName {
        self.add_fname_with_number(slice, 0)
    }

    /// Add an `FName` with number
    pub fn add_fname_with_number(&mut self, slice: &str, number: i32) -> FName {
        let index = self.add_name_reference(slice.to_string(), false);
        self.create_fname(index, number)
    }

    /// Returns if the name map is empty
    pub fn is_empty(&self) -> bool {
        self.name_map_index_list.is_empty()
    }
}

impl CyclicSharedResource<NameMap> for NameMap {
    fn on_cloned(&mut self, new_me: &SharedResourceWeakRef<NameMap>) {
        self.self_ref = new_me.clone();
    }
}

impl PartialEq for NameMap {
    fn eq(&self, other: &Self) -> bool {
        self.name_map_lookup == other.name_map_lookup
            && self.name_map_index_list == other.name_map_index_list
    }
}

impl Eq for NameMap {}
