use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::game_mod::SelectedVersion;
use crate::game_path_helpers::verify_install_path;
use crate::version::Version;
use crate::{GamePlatform, ModLoaderAppData};

#[derive(Serialize, Deserialize, Debug)]
struct ModLoaderConfig {
    selected_game_platform: Option<GamePlatform>,
}

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
    // TODO: make this a non-Option at some point
    force_latest: Option<bool>,
    priority: u16,
    enabled: bool,
    version: String,
}

pub(crate) fn load_modloader_config(data: &mut ModLoaderAppData) {
    let data_dir = BaseDirs::new().map(|e| PathBuf::from(e.data_dir()));
    if let Some(data_dir) = data_dir {
        let config_path = data_dir.join(data.config_dir).join("conf.json");
        if config_path.is_file() {
            let config_str = fs::read_to_string(config_path).unwrap();
            let config: ModLoaderConfig = serde_json::from_str(&config_str).unwrap_or_else(|_| {
                error!("Failed to parse conf.json");
                panic!();
            });

            if config.selected_game_platform.is_none() {
                if !data.set_game_platform(GamePlatform::Steam) {
                    let first_platform = data.install_managers.keys().next().unwrap().clone();
                    data.set_game_platform(first_platform);
                }
            } else {
                data.set_game_platform(config.selected_game_platform.unwrap());
            }
        }
    }
}

pub(crate) fn load_config(data: &mut ModLoaderAppData, game_name: &str) {
    let config_path = data.data_path.as_ref().unwrap().join("modconfig.json");
    if config_path.is_file() {
        let config_str = fs::read_to_string(config_path).unwrap();
        let config: ModConfig = serde_json::from_str(&config_str).unwrap_or_else(|_| {
            error!("Failed to parse modconfig.json");
            panic!();
        });

        data.refuse_mismatched_connections = config.refuse_mismatched_connections;

        let install_path = PathBuf::from(config.install_path);
        if verify_install_path(&install_path, game_name) {
            data.install_path = Some(install_path);
        }

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

            if !mod_config.force_latest.unwrap_or(true) {
                let config_version = Version::try_from(&mod_config.version);
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
    }
}

pub(crate) fn write_modloader_config(data: &mut ModLoaderAppData) {
    let data_dir = BaseDirs::new().map(|e| PathBuf::from(e.data_dir()));
    if let Some(data_dir) = data_dir {
        let config_path = data_dir.join(data.config_dir);
        fs::create_dir_all(config_path.clone()).unwrap();

        let config = ModLoaderConfig {
            selected_game_platform: data.selected_game_platform,
        };
        println!("Saving: {:?}", config);
        fs::write(
            config_path.join("conf.json"),
            serde_json::to_string(&config).unwrap(),
        )
        .unwrap();
    }
}

pub(crate) fn write_config(data: &mut ModLoaderAppData) {
    let config_path = data.data_path.as_ref().unwrap().join("modconfig.json");
    let mut config = ModConfig {
        install_path: data
            .install_path
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned(),
        refuse_mismatched_connections: data.refuse_mismatched_connections,
        current: ModsConfigData {
            mods: HashMap::new(),
        },
        profiles: HashMap::new(),
    };

    for (mod_id, game_mod) in data.game_mods.iter() {
        let mod_config = ModConfigData {
            force_latest: Some(match game_mod.selected_version {
                SelectedVersion::Latest(_) => true,
                SelectedVersion::LatestIndirect(_) => true,
                SelectedVersion::Specific(_) => false,
            }),
            priority: 0,
            enabled: game_mod.enabled,
            version: game_mod.selected_version.unwrap().to_string(),
        };

        config.current.mods.insert(mod_id.to_owned(), mod_config);
    }

    let config_str = serde_json::to_string(&config).unwrap();
    fs::write(config_path, config_str).unwrap();
}
