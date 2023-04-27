//! Unversioned properties ancestry

use crate::types::FName;

/// Unversioned properties ancestry
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ancestry {
    /// Ancestry array, last element is immediate parent
    pub ancestry: Vec<FName>,
}

impl Ancestry {
    /// Create a new `Ancestry` instance
    pub fn new(parent: FName) -> Self {
        Ancestry {
            ancestry: vec![parent],
        }
    }

    /// Gets immediate parent if one exists
    pub fn get_parent(&self) -> Option<&FName> {
        self.ancestry.last()
    }

    /// Clones this ancestry and adds a new immediate parent to the new one
    pub fn with_parent(&self, parent: FName) -> Self {
        let mut new_ancestry = self.clone();
        new_ancestry.ancestry.push(parent);
        new_ancestry
    }

    /// Clones this ancestry and traverses the ancestry list up
    pub fn without_parent(&self) -> Self {
        let mut new_ancestry = self.clone();
        new_ancestry.ancestry.pop();
        new_ancestry
    }
}
