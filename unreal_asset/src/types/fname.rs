//! FName is used to store most of the Strings in UE4.
//!
//! They are represented by an index+instance number inside a string table inside the asset file.

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    asset::name_map::NameMap,
    containers::{indexed_map::IndexedMap, shared_resource::SharedResource},
};

use std::hash::Hash;

/// FName name type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum EMappedNameType {
    /// Package-level name table
    Package,
    /// Container-level name table
    Container,
    /// Global name table
    Global,
}

/// FName is used to store most of the Strings in UE4.
///
/// They are represented by an index+instance number inside a string table inside the asset file.
#[derive(Debug, Clone)]
pub enum FName {
    /// Backed FName that is part of a namemap
    Backed {
        /// FName name map index
        index: i32,
        /// FName instance number
        number: i32,
        /// FName type
        ///
        /// Always [`EMappedNameType::Package`] for non-Zen assets
        ty: EMappedNameType,
        /// Namemap which this FName belongs to
        name_map: SharedResource<NameMap>,
    },
    /// Dummy FName that is not backed by any namemap, trying to serialize this will result in an `FNameError`
    Dummy {
        /// FName value
        value: String,
        /// FName instance number
        number: i32,
    },
}

/// Get implementer serialized name
pub trait ToSerializedName {
    /// Convert to serialized name
    ///
    /// # Warning
    ///
    /// This function is dangerous to call when a mutable reference to a name map exists
    /// Doing so may result in a panic
    fn to_serialized_name(&self) -> String;
}

impl FName {
    /// FName index bits
    pub const INDEX_BITS: u32 = 30;
    /// FName index mask
    pub const INDEX_MASK: u32 = (1u32 << Self::INDEX_BITS).overflowing_sub(1).0;
    /// FName type mask
    pub const TYPE_MASK: u32 = !Self::INDEX_MASK;
    /// FName type shift
    pub const TYPE_SHIFT: u32 = Self::INDEX_BITS;

    /// Create a new `FName` instance with an index
    pub fn new(index: i32, number: i32, name_map: SharedResource<NameMap>) -> Self {
        FName::Backed {
            index,
            number,
            ty: EMappedNameType::Package,
            name_map,
        }
    }

    /// Create a new `FName` instance with an index and type
    pub fn new_with_type(
        index: i32,
        number: i32,
        ty: EMappedNameType,
        name_map: SharedResource<NameMap>,
    ) -> Self {
        FName::Backed {
            index,
            number,
            ty,
            name_map,
        }
    }

    /// Create a new "dummy" `FName` instance from a slice and an index
    pub fn new_dummy(value: String, number: i32) -> Self {
        FName::Dummy { value, number }
    }

    /// Create a new "dummy" `FName` instance from a slice with an index of 0
    pub fn from_slice(value: &str) -> Self {
        FName::new_dummy(value.to_string(), 0)
    }

    /// Get this FName content
    pub fn get_content(&self) -> String {
        // todo: return string ref
        match self {
            FName::Backed {
                index,
                number: _,
                ty: _,
                name_map,
            } => name_map.get_ref().get_name_reference(*index),
            FName::Dummy { value, number: _ } => value.clone(),
        }
    }
}

impl PartialEq for FName {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                FName::Backed {
                    index: a_index,
                    number: a_number,
                    ty: _,
                    name_map: _,
                },
                FName::Backed {
                    index: b_index,
                    number: b_number,
                    ty: _,
                    name_map: _,
                },
            ) => a_index == b_index && a_number == b_number,
            (
                FName::Dummy {
                    value: a_value,
                    number: a_number,
                },
                FName::Dummy {
                    value: b_value,
                    number: b_number,
                },
            ) => a_value == b_value && a_number == b_number,
            _ => false,
        }
    }
}

impl Eq for FName {}

impl Hash for FName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            FName::Backed {
                index,
                number,
                ty: _,
                name_map: _,
            } => {
                index.hash(state);
                number.hash(state);
            }
            FName::Dummy { value, number } => {
                value.hash(state);
                number.hash(state);
            }
        }
    }
}

impl Default for FName {
    fn default() -> Self {
        FName::Dummy {
            value: String::default(),
            number: i32::default(),
        }
    }
}

/// A trait that can be implemented for structs that contain an FName
///
/// This trait will be typically used to traverse the whole asset FName tree
/// and rebuild the name map.
pub trait FNameContainer {
    /// Traverse this fname container
    ///
    /// Traverse function must get called for each FName in this container
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F);
}

impl FNameContainer for FName {
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F) {
        traverse(self);
    }
}

impl<T: FNameContainer> FNameContainer for Vec<T> {
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F) {
        for e in self.iter_mut() {
            e.traverse_fnames(traverse);
        }
    }
}

impl<T: FNameContainer> FNameContainer for Box<T> {
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F) {
        self.as_mut().traverse_fnames(traverse)
    }
}

// todo: fix indexedmap fname key access
impl<K, V> FNameContainer for IndexedMap<K, V>
where
    K: Eq + Hash + FNameContainer,
    V: Eq + Hash + FNameContainer + Clone,
{
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F) {
        *self = self
            .clone()
            .into_iter()
            .map(|(_, mut key, mut value)| {
                key.traverse_fnames(traverse);
                value.traverse_fnames(traverse);
                (key, value)
            })
            .collect::<IndexedMap<K, V>>();
    }
}

impl FNameContainer for IndexedMap<FName, super::PackageIndex> {
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F) {
        *self = self
            .clone()
            .into_iter()
            .map(|(_, mut key, value)| {
                key.traverse_fnames(traverse);
                (key, value)
            })
            .collect::<IndexedMap<_, _>>();
    }
}

impl<T: FNameContainer> FNameContainer for Option<T> {
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F) {
        if let Some(e) = self {
            e.traverse_fnames(traverse);
        }
    }
}

impl<T: FNameContainer> FNameContainer for SharedResource<T> {
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F) {
        let mut mut_self = self.get_mut();
        mut_self.traverse_fnames(traverse);
    }
}

impl<T: ordered_float::Float> FNameContainer for ordered_float::OrderedFloat<T> {
    fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, _: &mut F) {}
}

macro_rules! dummy_container_impl {
    ($($ty:ty),*) => {
        $(
            impl FNameContainer for $ty {
                fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, _: &mut F) {}
            }
        )*
    };
}

macro_rules! tuple_container_impl {
    ($($name:ident),*) => {
        impl<$($name:FNameContainer),*> FNameContainer for ($($name,)*)
        {
            fn traverse_fnames<F: FnMut(&mut FName)>(&mut self, traverse: &mut F) {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                $($name.traverse_fnames(traverse);)*
            }
        }
    };
}

dummy_container_impl!(
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    f32,
    f64,
    bool,
    String,
    &str,
    super::Guid
);
tuple_container_impl!(A);
tuple_container_impl!(A, B);
tuple_container_impl!(A, B, C);
tuple_container_impl!(A, B, C, D);
tuple_container_impl!(A, B, C, D, E);
tuple_container_impl!(A, B, C, D, E, G);
tuple_container_impl!(A, B, C, D, E, G, H);
tuple_container_impl!(A, B, C, D, E, G, H, I);
tuple_container_impl!(A, B, C, D, E, G, H, I, J);
tuple_container_impl!(A, B, C, D, E, G, H, I, J, K);
tuple_container_impl!(A, B, C, D, E, G, H, I, J, K, L);
tuple_container_impl!(A, B, C, D, E, G, H, I, J, K, L, M);
