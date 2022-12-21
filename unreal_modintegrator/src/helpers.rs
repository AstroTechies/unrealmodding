use std::fs::File;
use std::io::Cursor;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;

use unreal_asset::{engine_version::EngineVersion, Asset};
use unreal_pak::{PakMemory, PakReader};

use crate::{error::IntegrationError, Error};

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
    integrated_pak: &mut PakMemory,
    game_paks: &mut [PakReader<File>],
    mod_paks: &mut [PakReader<File>],
    name: &String,
    version: EngineVersion,
) -> Result<Asset, Error> {
    if let Ok(asset) = read_asset(
        |name| Ok(integrated_pak.get_entry(name).cloned()),
        version,
        name,
    ) {
        return Ok(asset);
    }

    if let Some(mod_pak_index) = find_asset(mod_paks, name) {
        return read_asset(
            |name| {
                mod_paks[mod_pak_index].read_entry(name).map_or_else(
                    |err| {
                        if matches!(err.kind, unreal_pak::error::PakErrorKind::EntryNotFound(_)) {
                            Ok(None)
                        } else {
                            Err(err.into())
                        }
                    },
                    |data| Ok(Some(data)),
                )
            },
            version,
            name,
        );
    }

    let game_pak_index = find_asset(game_paks, name)
        .ok_or_else(|| IntegrationError::asset_not_found(name.clone()))?;

    read_asset(
        |name| {
            game_paks[game_pak_index].read_entry(name).map_or_else(
                |err| {
                    if matches!(err.kind, unreal_pak::error::PakErrorKind::EntryNotFound(_)) {
                        Ok(None)
                    } else {
                        Err(err.into())
                    }
                },
                |data| Ok(Some(data)),
            )
        },
        version,
        name,
    )
}

pub fn find_asset(paks: &mut [PakReader<File>], name: &String) -> Option<usize> {
    for (i, pak) in paks.iter().enumerate() {
        if pak.contains_entry(name) {
            return Some(i);
        }
    }
    None
}

pub fn read_asset<F>(
    mut read_fn: F,
    engine_version: EngineVersion,
    name: &String,
) -> Result<Asset, Error>
where
    F: FnMut(&String) -> Result<Option<Vec<u8>>, Error>,
{
    let uexp = read_fn(
        &Path::new(name)
            .with_extension("uexp")
            .to_str()
            .unwrap()
            .to_string(),
    )?;
    let uasset = read_fn(name)?.ok_or_else(|| IntegrationError::asset_not_found(name.clone()))?;

    let mut asset = Asset::new(uasset, uexp);
    asset.set_engine_version(engine_version);
    asset.parse_data()?;
    Ok(asset)
}

pub fn write_asset(pak: &mut PakMemory, asset: &Asset, name: &String) -> Result<(), Error> {
    let mut uasset_cursor = Cursor::new(Vec::new());
    let mut uexp_cursor = match asset.use_separate_bulk_data_files {
        true => Some(Cursor::new(Vec::new())),
        false => None,
    };
    asset.write_data(&mut uasset_cursor, uexp_cursor.as_mut())?;

    pak.set_entry(name.clone(), uasset_cursor.into_inner());

    if let Some(cursor) = uexp_cursor {
        pak.set_entry(
            Path::new(name)
                .with_extension("uexp")
                .to_str()
                .unwrap()
                .to_string(),
            cursor.into_inner(),
        )
    }
    Ok(())
}
