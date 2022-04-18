use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::AppData;

#[derive(Serialize, Deserialize, Debug)]
struct ModConfig {
    install_path: String,
    refuse_mismatched_connections: bool,
    current: ModsConfigData,
    profiles: HashMap<String, ModsConfigData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModsConfigData {
    mods: HashMap<String, ModConfigData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModConfigData {
    force_latest: Option<bool>,
    priority: u16,
    enabled: bool,
    version: String,
}

pub(crate) fn load_config(data: &mut AppData) {
    let config_path = data.data_path.as_ref().unwrap().join("modconfig.json");
    if config_path.is_file() {
        let config_str = fs::read_to_string(config_path).unwrap();
        let config: ModConfig = serde_json::from_str(&config_str).unwrap();
    }
}
