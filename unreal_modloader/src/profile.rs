use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub mods: HashMap<String, ProfileMod>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProfileMod {
    #[serde(default = "crate::default_true")]
    pub force_latest: bool,
    pub priority: u16,
    pub version: String,
}

pub fn parse_profile_config(profile_config: Value) -> Result<Vec<Profile>, serde_json::Error> {
    if profile_config.is_array() {
        Ok(serde_json::from_value(profile_config)?)
    } else {
        // TODO: legacy, remove at some point
        // try to interpret as map, if fails return error
        let profiles: HashMap<String, LegacyProfile> = serde_json::from_value(profile_config)?;

        Ok(profiles
            .into_iter()
            .map(|(name, profile)| Profile {
                name,
                mods: profile
                    .mods
                    .into_iter()
                    .filter(|(_, profile_mod)| profile_mod.enabled)
                    .map(|(mod_id, profile_mod)| {
                        (
                            mod_id,
                            ProfileMod {
                                force_latest: profile_mod.force_latest,
                                priority: profile_mod.priority,
                                version: profile_mod.version,
                            },
                        )
                    })
                    .collect(),
            })
            .collect())
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
struct LegacyProfile {
    mods: HashMap<String, LegacyProfileMod>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
struct LegacyProfileMod {
    #[serde(default = "crate::default_true")]
    force_latest: bool,
    enabled: bool,
    priority: u16,
    version: String,
}
