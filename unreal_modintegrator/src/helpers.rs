use std::{
    io::{self, ErrorKind},
    path::Path,
};

use lazy_static::lazy_static;
use regex::Regex;

use unreal_asset::Asset;
use unreal_pak::PakFile;

use crate::find_asset;
use crate::read_asset;

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new(r"^/Game/").unwrap();
}

pub fn game_to_absolute(game_name: &str, path: &str) -> Option<String> {
    if !GAME_REGEX.is_match(path) {
        return None;
    }

    let path_str = GAME_REGEX
        .replace(path, String::from(game_name) + "/Content/")
        .to_string();
    let path = Path::new(&path_str);
    match path.extension() {
        Some(_) => Some(path_str),
        None => path
            .with_extension("uasset")
            .to_str()
            .map(|e| e.to_string()),
    }
}

pub fn get_asset(
    integrated_pak: &mut PakFile,
    game_paks: &mut [PakFile],
    mod_paks: &mut [PakFile],
    name: &String,
    version: i32,
) -> Result<Asset, io::Error> {
    if let Ok(asset) = read_asset(integrated_pak, version, name) {
        return Ok(asset);
    }

    if let Some(mod_asset) = find_asset(mod_paks, name) {
        return read_asset(&mut mod_paks[mod_asset], version, name)
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()));
    }

    let original_asset = find_asset(game_paks, name)
        .ok_or_else(|| io::Error::new(ErrorKind::Other, format!("No such asset {}", name)))?;

    read_asset(&mut game_paks[original_asset], version, name)
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))
}
