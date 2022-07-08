use assets::{COPY_OVER, INTEGRATOR_STATICS_ASSET, LIST_OF_MODS_ASSET, METADATA_JSON};
#[cfg(feature = "bulk_data")]
use assets::{INTEGRATOR_STATICS_BULK, LIST_OF_MODS_BULK};

use error::IntegrationError;
use metadata::{Metadata, SyncMode};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Cursor;
use std::path::Path;
use unreal_asset::exports::data_table_export::DataTable;
use unreal_asset::exports::{Export, ExportBaseTrait};
use unreal_asset::properties::int_property::{BoolProperty, ByteProperty};
use unreal_asset::properties::str_property::StrProperty;
use unreal_asset::properties::struct_property::StructProperty;
use unreal_asset::properties::{Property, PropertyDataTrait};
use unreal_asset::unreal_types::FName;
use unreal_pak::pakversion::PakVersion;

mod assets;
pub mod error;
pub mod metadata;
use serde_json::Value;
use unreal_asset::Asset;
use unreal_pak::{PakFile, PakRecord};

use crate::error::Error;

pub trait IntegratorInfo {}

pub const INTEGRATOR_PAK_FILE_NAME: &str = "900-ModIntegrator_P.pak";

#[allow(clippy::type_complexity)]
pub trait IntegratorConfig<'data, T, E: std::error::Error> {
    fn get_data(&self) -> &'data T;
    fn get_handlers(
        &self,
    ) -> HashMap<
        String,
        Box<
            dyn FnMut(
                &T,
                &mut PakFile,
                &mut Vec<PakFile>,
                &mut Vec<PakFile>,
                Vec<&Value>,
            ) -> Result<(), E>,
        >,
    >;

    const GAME_NAME: &'static str;
    const INTEGRATOR_VERSION: &'static str;
    const ENGINE_VERSION: i32;
}

pub fn find_asset(paks: &mut [PakFile], name: &String) -> Option<usize> {
    for (i, pak) in paks.iter().enumerate() {
        if pak.records.contains_key(name) {
            return Some(i);
        }
    }
    None
}

pub fn read_asset(pak: &mut PakFile, engine_version: i32, name: &String) -> Result<Asset, Error> {
    let uexp = pak
        .get_record(
            &Path::new(name)
                .with_extension("uexp")
                .to_str()
                .unwrap()
                .to_string(),
        )
        .ok()
        .and_then(|e| e.data.clone());

    let uasset = pak.get_record(name)?.data.as_ref().unwrap().clone();
    let mut asset = Asset::new(uasset, uexp);
    asset.engine_version = engine_version;
    asset.parse_data()?;
    Ok(asset)
}

fn read_in_memory(
    uasset: Vec<u8>,
    uexp: Option<Vec<u8>>,
    engine_version: i32,
) -> Result<Asset, Error> {
    let mut asset = Asset::new(uasset, uexp);
    asset.engine_version = engine_version;
    asset.parse_data()?;
    Ok(asset)
}

pub fn write_asset(pak: &mut PakFile, asset: &Asset, name: &String) -> Result<(), Error> {
    let mut uasset_cursor = Cursor::new(Vec::new());
    let mut uexp_cursor = match asset.use_separate_bulk_data_files {
        true => Some(Cursor::new(Vec::new())),
        false => None,
    };
    asset.write_data(&mut uasset_cursor, uexp_cursor.as_mut())?;

    let record = PakRecord::new(
        name.clone(),
        uasset_cursor.get_ref().to_owned(),
        unreal_pak::CompressionMethod::Zlib,
    )?;
    pak.add_record(record)?;

    if let Some(cursor) = uexp_cursor {
        let uexp_record = PakRecord::new(
            Path::new(name)
                .with_extension("uexp")
                .to_str()
                .unwrap()
                .to_string(),
            cursor.get_ref().to_owned(),
            unreal_pak::CompressionMethod::Zlib,
        )?;
        pak.add_record(uexp_record)?;
    }
    Ok(())
}

fn bake_mod_data(asset: &mut Asset, mods: &Vec<Metadata>) -> Result<(), Error> {
    let data_table_export = asset
        .exports
        .iter()
        .filter_map(|e| match e {
            Export::DataTableExport(e) => Some(e),
            _ => None,
        })
        .next();
    if data_table_export.is_none() {
        return Err(IntegrationError::corrupted_starter_pak().into());
    }
    let data_table_export = data_table_export.unwrap();

    let tab = data_table_export
        .table
        .data
        .get(0)
        .ok_or_else(IntegrationError::corrupted_starter_pak)?;
    let struct_type = tab.struct_type.clone();
    let columns: Vec<FName> = tab.value.iter().map(|e| e.get_name()).collect();
    let mut duplication_indices = HashMap::new();
    let mut new_table: Vec<StructProperty> = Vec::new();

    for mod_data in mods {
        asset.add_name_reference(mod_data.mod_id.clone(), false);

        let coded_sync_mode = match mod_data.sync.unwrap_or(SyncMode::ServerAndClient) {
            SyncMode::ServerAndClient => "SyncMode::NewEnumerator3",
            SyncMode::ServerOnly => "SyncMode::NewEnumerator2",
            SyncMode::ClientOnly => "SyncMode::NewEnumerator1",
            SyncMode::None => "SyncMode::NewEnumerator0",
        };

        let rows: Vec<Property> = vec![
            StrProperty {
                name: columns[0].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.name.clone()),
            }
            .into(),
            StrProperty {
                name: columns[1].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.author.clone().unwrap_or_default()),
            }
            .into(),
            StrProperty {
                name: columns[2].clone(),
                property_guid: None,
                duplication_index: 9,
                value: Some(mod_data.description.clone().unwrap_or_default()),
            }
            .into(),
            StrProperty {
                name: columns[3].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.mod_version.clone()),
            }
            .into(),
            StrProperty {
                name: columns[4].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.game_build.clone().unwrap_or_default()),
            }
            .into(),
            ByteProperty {
                name: columns[5].clone(),
                property_guid: None,
                duplication_index: 0,
                enum_type: Some(asset.add_name_reference(String::from("SyncMode"), false) as i64),
                byte_type: unreal_asset::properties::int_property::ByteType::Long,
                value: asset.add_name_reference(String::from(coded_sync_mode), false) as i64,
            }
            .into(),
            StrProperty {
                name: columns[6].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.homepage.clone().unwrap_or_default()),
            }
            .into(),
            BoolProperty {
                name: columns[7].clone(),
                property_guid: None,
                duplication_index: 0,
                value: true, // optional modids?
            }
            .into(),
        ];

        let duplication_index = duplication_indices
            .entry(mod_data.mod_id.clone())
            .or_insert_with(|| 0);
        new_table.push(StructProperty {
            name: FName::new(mod_data.mod_id.clone(), 0),
            struct_type: struct_type.clone(),
            struct_guid: None,
            property_guid: None,
            duplication_index: *duplication_index,
            serialize_none: false,
            value: rows,
        });
        *duplication_index += 1;
    }

    let data_table_export = asset
        .exports
        .iter_mut()
        .filter_map(|e| match e {
            Export::DataTableExport(e) => Some(e),
            _ => None,
        })
        .next();
    if data_table_export.is_none() {
        return Err(IntegrationError::corrupted_starter_pak().into());
    }
    let data_table_export = data_table_export.unwrap();

    data_table_export.table = DataTable::new(new_table);

    Ok(())
}

fn bake_integrator_data(
    asset: &mut Asset,
    integrator_version: String,
    refuse_mismatched_connections: bool,
) -> Result<(), Error> {
    if asset.exports.len() != 4 {
        return Err(IntegrationError::corrupted_starter_pak().into());
    }

    let properties: Vec<Property> = Vec::from([
        StrProperty {
            name: FName::from_slice("IntegratorVersion"),
            property_guid: None,
            duplication_index: 0,
            value: Some(integrator_version),
        }
        .into(),
        BoolProperty {
            name: FName::from_slice("RefuseMismatchedConnections"),
            property_guid: None,
            duplication_index: 0,
            value: refuse_mismatched_connections,
        }
        .into(),
    ]);

    let export = asset
        .exports
        .iter_mut()
        .find(|e| e.get_base_export().object_name.content == "Default__IntegratorStatics_BP_C");
    if export.is_none() {
        return Err(IntegrationError::corrupted_starter_pak().into());
    }

    let export = export.unwrap();
    match export {
        Export::NormalExport(e) => {
            e.properties = properties;
            Ok(())
        }
        _ => Err(IntegrationError::corrupted_starter_pak().into()),
    }
}

pub fn integrate_mods<
    'data,
    T: 'data,
    E: 'static + std::error::Error + Send,
    C: IntegratorConfig<'data, T, E>,
>(
    integrator_config: &C,
    paks_path: &Path,
    game_path: &Path,
    refuse_mismatched_connections: bool,
) -> Result<(), Error> {
    let mods_dir = fs::read_dir(paks_path)?;
    let mod_files: Vec<File> = mods_dir
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .into_string()
                .map(|e| e.ends_with("_P.pak") && e != INTEGRATOR_PAK_FILE_NAME)
                .unwrap_or(false)
        })
        .map(|e| File::open(&e.path()))
        .filter_map(|e| e.ok())
        .collect();

    let game_dir = fs::read_dir(game_path)?;
    let game_files: Vec<File> = game_dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|e| e == "pak").unwrap_or(false))
        .filter_map(|e| File::open(&e.path()).ok())
        .collect();
    if game_files.is_empty() {
        return Err(IntegrationError::game_not_found().into());
    }

    let mut mods = Vec::new();
    let mut mod_paks = Vec::new();
    let mut optional_mods_data = Vec::new();
    for mod_file in &mod_files {
        let mut pak = PakFile::reader(mod_file);
        pak.load_records()?;

        let record = pak
            .get_record(&String::from("metadata.json"))?
            .data
            .as_ref()
            .unwrap();
        let metadata: Metadata = serde_json::from_slice(record)?;
        mods.push(metadata.clone());

        let optional_metadata: Value = serde_json::from_slice(record)?;
        optional_mods_data.push(optional_metadata);

        mod_paks.push(pak);
    }

    if !mods.is_empty() {
        let path = Path::new(paks_path).join(INTEGRATOR_PAK_FILE_NAME);
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        let file = OpenOptions::new().append(true).open(&path)?;
        let mut generated_pak =
            PakFile::writer(PakVersion::PakFileVersionFnameBasedCompressionMethod, &file);

        #[cfg(not(feature = "bulk_data"))]
        let list_of_mods_bulk = None;
        #[cfg(feature = "bulk_data")]
        let list_of_mods_bulk = Some(LIST_OF_MODS_BULK.to_vec());

        let mut list_of_mods = read_in_memory(
            LIST_OF_MODS_ASSET.to_vec(),
            list_of_mods_bulk,
            C::ENGINE_VERSION,
        )?;
        bake_mod_data(&mut list_of_mods, &mods)?;
        write_asset(
            &mut generated_pak,
            &list_of_mods,
            &(C::GAME_NAME.to_owned() + "/Content/Integrator/ListOfMods.uasset"),
        )?;

        #[cfg(not(feature = "bulk_data"))]
        let integrator_statics_bulk = None;
        #[cfg(feature = "bulk_data")]
        let integrator_statics_bulk = Some(INTEGRATOR_STATICS_BULK.to_vec());

        let mut integrator_statics = read_in_memory(
            INTEGRATOR_STATICS_ASSET.to_vec(),
            integrator_statics_bulk,
            C::ENGINE_VERSION,
        )?;

        bake_integrator_data(
            &mut integrator_statics,
            C::INTEGRATOR_VERSION.to_owned(),
            refuse_mismatched_connections,
        )?;
        write_asset(
            &mut generated_pak,
            &integrator_statics,
            &(C::GAME_NAME.to_owned() + "/Content/Integrator/IntegratorStatics_BP.uasset"),
        )?;

        let metadata_record = PakRecord::new(
            String::from("metadata.json"),
            METADATA_JSON.to_vec(),
            unreal_pak::CompressionMethod::Zlib,
        )?;
        generated_pak.add_record(metadata_record)?;

        for entry in &COPY_OVER {
            let record = PakRecord::new(
                C::GAME_NAME.to_owned() + "/Content/Integrator/" + entry.1,
                entry.0.to_vec(),
                unreal_pak::CompressionMethod::Zlib,
            )?;
            generated_pak.add_record(record)?;
        }

        let mut game_paks = Vec::new();
        for game_file in &game_files {
            let mut pak = PakFile::reader(game_file);
            pak.load_records()?;
            game_paks.push(pak);
        }

        for (name, mut exec) in integrator_config.get_handlers() {
            let all_mods: Vec<&Value> = optional_mods_data
                .iter()
                .filter_map(|e| match e[name.clone()] != Value::Null {
                    true => Some(&e[name.clone()]),
                    false => None,
                })
                .collect();
            exec(
                integrator_config.get_data(),
                &mut generated_pak,
                &mut game_paks,
                &mut mod_paks,
                all_mods,
            )
            .map_err(|e| Error::other(Box::new(e)))?;
        }

        generated_pak.write()?;
        file.sync_data()?;
    }

    Ok(())
}
