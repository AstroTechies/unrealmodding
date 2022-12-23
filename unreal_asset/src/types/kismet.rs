use super::{FName, PackageIndex};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FieldPath {
    pub path: Vec<FName>,
    pub resolved_owner: PackageIndex,
}

impl FieldPath {
    pub fn new(path: Vec<FName>, resolved_owner: PackageIndex) -> Self {
        FieldPath {
            path,
            resolved_owner,
        }
    }
}
