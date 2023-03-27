use std::collections::HashMap;
use std::fs;

use log::{debug, error, warn};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::game_mod::SelectedVersion;
use crate::profile::parse_profile_config;
use crate::ModLoaderAppData;

#[derive(Serialize, Deserialize, Debug)]
struct ModConfig {
    selected_game_platform: Option<String>,
    refuse_mismatched_connections: bool,
    current: ModsConfigData,
    #[serde(default)]
    trusted_mods: Vec<String>,
    profiles: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModsConfigData {
    mods: HashMap<String, ModConfigData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModConfigData {
    #[serde(default = "crate::default_true")]
    force_latest: bool,
    priority: u16,
    enabled: bool,
    version: String,
}

pub(crate) fn load_config(data: &mut ModLoaderAppData) {
    macro_rules! bail {
        () => {{
            warn!("Error reading config file!");
            if !data.set_game_platform("Steam") {
                let first_platform = data.install_managers.keys().next().unwrap();
                data.set_game_platform(first_platform);
            }
            return;
        }};
    }

    let config_path = data.mods_path.as_ref().unwrap().join("modconfig.json");

    if !config_path.is_file() {
        bail!()
    }

    let config: ModConfig = match serde_json::from_str(&fs::read_to_string(&config_path).unwrap()) {
        Ok(config) => config,
        Err(err) => {
            error!("{}", err);
            let _ = fs::remove_file(&config_path);
            bail!();
        }
    };

    data.refuse_mismatched_connections = config.refuse_mismatched_connections;

    for (mod_id, mod_config) in config.current.mods.iter() {
        let game_mod = data.game_mods.get_mut(mod_id);
        if game_mod.is_none() {
            warn!(
                "Mod {} referenced in modconfig.json is not installed",
                mod_id
            );
            continue;
        }
        let game_mod = game_mod.unwrap();

        game_mod.enabled = mod_config.enabled;

        if !mod_config.force_latest {
            let config_version = Version::parse(&mod_config.version);
            if config_version.is_err() {
                warn!(
                    "Failed to parse version {} for mod {}",
                    mod_config.version, mod_id
                );
                continue;
            }

            game_mod.selected_version = SelectedVersion::Specific(config_version.unwrap());
        }
    }

    data.trusted_mods = config
        .trusted_mods
        .iter()
        .filter_map(|e| hex::decode(e).ok())
        .collect::<Vec<_>>();

    data.profiles = match parse_profile_config(config.profiles) {
        Ok(profiles) => profiles,
        Err(err) => {
            // not such a bad error
            error!("{}", err);
            Vec::new()
        }
    };

    if let Some(ref selected_game_platform) = config.selected_game_platform {
        data.set_game_platform(selected_game_platform);
    } else if !data.set_game_platform("Steam") {
        let first_platform = data.install_managers.keys().next().unwrap();
        data.set_game_platform(first_platform);
    }

    debug!("Loaded config");
}

pub(crate) fn write_config(data: &mut ModLoaderAppData) {
    let config_path = data.mods_path.as_ref().unwrap().join("modconfig.json");
    let mut config = ModConfig {
        selected_game_platform: data.selected_game_platform.clone(),
        refuse_mismatched_connections: data.refuse_mismatched_connections,
        current: ModsConfigData {
            mods: HashMap::new(),
        },
        trusted_mods: data
            .trusted_mods
            .iter()
            .map(hex::encode)
            .collect::<Vec<_>>(),
        profiles: serde_json::to_value(data.profiles.clone()).unwrap(),
    };

    for (mod_id, game_mod) in data.game_mods.iter() {
        let mod_config = ModConfigData {
            force_latest: game_mod.selected_version.is_latest(),
            priority: 0,
            enabled: game_mod.enabled,
            version: game_mod.selected_version.clone().unwrap().to_string(),
        };

        config.current.mods.insert(mod_id.to_owned(), mod_config);
    }

    let config_str = serde_json::to_string(&config).unwrap();
    fs::write(config_path, config_str).unwrap();

    debug!("Wrote config");
}
