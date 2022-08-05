#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub changelog: String,
}

impl UpdateInfo {
    pub fn new(version: String, changelog: String) -> Self {
        UpdateInfo { version, changelog }
    }
}
