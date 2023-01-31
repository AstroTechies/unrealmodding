use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Profile {
    mods: HashMap<String, ProfileMod>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProfileMod {
    #[serde(default = "crate::default_true")]
    force_latest: bool,
    priority: u16,
    version: String,
}
