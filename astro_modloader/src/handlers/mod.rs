use std::{
    io::{self, ErrorKind},
    path::Path,
};

use lazy_static::lazy_static;

use regex::Regex;
use unreal_asset::Asset;
use unreal_modintegrator::{find_asset, read_asset};
use unreal_pak::PakFile;

pub(crate) mod item_list_entries;
pub(crate) mod linked_actor_components;
pub(crate) mod mission_trailheads;
pub mod persistent_actors;

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new(r"^/Game/").unwrap();
}

static MAP_PATHS: [&str; 3] = [
    "Astro/Content/Maps/Staging_T2.umap",
    "Astro/Content/Maps/Staging_T2_PackedPlanets_Switch.umap",
    //"Astro/Content/Maps/TutorialMoon_Prototype_v2.umap", // Tutorial not integrated for performance
    "Astro/Content/Maps/test/BasicSphereT2.umap",
];

fn get_asset(
    integrated_pak: &mut PakFile,
    game_paks: &mut [PakFile],
    name: &String,
    version: i32,
) -> Result<Asset, io::Error> {
    if let Ok(asset) = read_asset(integrated_pak, version, name) {
        return Ok(asset);
    }
    let original_asset = find_asset(game_paks, name)
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "No such ass"))?;

    read_asset(&mut game_paks[original_asset], version, name)
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))
}

fn game_to_absolute(path: &str) -> Option<String> {
    if !GAME_REGEX.is_match(path) {
        return None;
    }

    let path_str = GAME_REGEX.replace(path, "Astro/Content/").to_string();
    let path = Path::new(&path_str);
    match path.extension() {
        Some(_) => Some(path_str),
        None => path
            .with_extension("uasset")
            .to_str()
            .map(|e| e.to_string()),
    }
}
