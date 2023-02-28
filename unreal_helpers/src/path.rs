//! Functions for working with Unreal paths

use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new(r"^/Game/").unwrap();
}

/// Turn an Unreal game path into an absolute path that can be used to access files on disk.
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
