use std::{collections::HashMap, io::Cursor};

use unreal_asset::{engine_version::EngineVersion, error::Error, Asset};

mod shared;

macro_rules! assets_folder {
    () => {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/duplicate_name_map_entries/"
        )
    };
}

const ASSET_FILE: &[u8] = include_bytes!(concat!(assets_folder!(), "BIOME_AzureWeald.uasset"));
const ASSET_BULK_FILE: &[u8] = include_bytes!(concat!(assets_folder!(), "BIOME_AzureWeald.uexp"));

#[test]
fn duplicate_name_map_entries() -> Result<(), Error> {
    let mut asset = Asset::new(Cursor::new(ASSET_FILE), Some(Cursor::new(ASSET_BULK_FILE)));
    asset.set_engine_version(EngineVersion::VER_UE4_25);

    asset.parse_data()?;

    let mut has_duplicates = false;

    let mut enumerated_entries = HashMap::new();

    for entry in asset.get_name_map_index_list() {
        if enumerated_entries.contains_key(entry) {
            has_duplicates = true;
            break;
        }

        enumerated_entries.insert(entry.clone(), true);
    }

    assert!(has_duplicates);
    assert!(shared::verify_all_exports_parsed(&asset));

    Ok(())
}
