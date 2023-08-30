use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

use unreal_asset::{engine_version::EngineVersion, reader::ArchiveTrait, Asset};
use unreal_pak::{PakMemory, PakReader};

use crate::{error::IntegrationError, Error};

pub fn get_asset(
    integrated_pak: &mut PakMemory,
    game_paks: &mut [PakReader<BufReader<File>>],
    mod_paks: &mut [PakReader<BufReader<File>>],
    name: &String,
    version: EngineVersion,
) -> Result<Asset<Cursor<Vec<u8>>>, Error> {
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

pub fn find_asset(paks: &mut [PakReader<BufReader<File>>], name: &String) -> Option<usize> {
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
) -> Result<Asset<Cursor<Vec<u8>>>, Error>
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

    Ok(Asset::new(
        Cursor::new(uasset),
        uexp.map(Cursor::new),
        engine_version,
        None,
    )?)
}

pub fn write_asset<C: std::io::Read + std::io::Seek>(
    pak: &mut PakMemory,
    asset: &Asset<C>,
    name: &String,
) -> Result<(), Error> {
    let mut uasset_cursor = Cursor::new(Vec::new());
    let mut uexp_cursor = match asset.use_event_driven_loader() {
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
