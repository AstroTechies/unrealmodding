use assets::{COPY_OVER, INTEGRATOR_STATICS_ASSET, LIST_OF_MODS_ASSET, METADATA_JSON};
use error::IntegrationError;
use metadata::{Metadata, SyncMode};
use std::collections::HashMap;
use std::fs;
use std::fs::{DirEntry, File, OpenOptions};
use std::io::Cursor;
use std::path::Path;
use unreal_asset::exports::data_table_export::DataTable;
use unreal_asset::exports::normal_export::NormalExport;
use unreal_asset::exports::Export;
use unreal_asset::properties::int_property::{BoolProperty, ByteProperty};
use unreal_asset::properties::object_property::ObjectProperty;
use unreal_asset::properties::str_property::StrProperty;
use unreal_asset::properties::struct_property::StructProperty;
use unreal_asset::properties::{Property, PropertyDataTrait};
use unreal_asset::unreal_types::FName;

mod assets;
pub mod error;
pub mod metadata;
use serde_json::Value;
use unreal_asset::{Asset, Import};
use unreal_pak::PakFile;

use crate::error::Error;

pub trait IntegratorInfo {}

pub trait IntegratorConfig<'data, T, E: std::error::Error> {
    fn get_data(&self) -> &'data T;
    fn get_handlers(
        &self,
    ) -> HashMap<
        String,
        Box<dyn FnMut(&T, &mut PakFile, &mut Vec<PakFile>, Vec<&Value>) -> Result<(), E>>,
    >;

    fn get_game_name(&self) -> String;
    fn get_integrator_version(&self) -> String;
    fn get_engine_version(&self) -> i32;
}

pub fn find_asset(paks: &mut Vec<PakFile>, name: &String) -> Option<Vec<u8>> {
    for pak in paks {
        pak.read_record(name).ok()?;
    }
    None
}

pub fn read_asset(pak: &mut PakFile, engine_version: i32, name: &String) -> Result<Asset, Error> {
    let uexp = pak.read_record(&name.replace(".uasset", ".uexp")).ok();
    let uasset = pak.read_record(name)?;
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

fn write_asset(pak: &mut PakFile, asset: &Asset, name: &String) -> Result<(), Error> {
    let mut uasset_cursor = Cursor::new(Vec::new());
    let mut uexp_cursor = match asset.use_separate_bulk_data_files {
        true => Some(Cursor::new(Vec::new())),
        false => None,
    };
    asset.write_data(&mut uasset_cursor, uexp_cursor.as_mut())?;

    pak.write_record(
        name,
        &uasset_cursor.get_ref().to_owned(),
        &unreal_pak::CompressionMethod::Zlib,
    )?;
    if let Some(cursor) = uexp_cursor {
        pak.write_record(
            &name.to_owned().replace(".uasset", ".uexp"),
            &cursor.get_ref().to_owned(),
            &unreal_pak::CompressionMethod::Zlib,
        )?;
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
        .ok_or::<Error>(IntegrationError::corrupted_starter_pak().into())?;
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

        let mut rows: Vec<Property> = Vec::new();

        rows.push(
            StrProperty {
                name: columns[0].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.name.clone()),
            }
            .into(),
        );

        rows.push(
            StrProperty {
                name: columns[1].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.author.clone().unwrap_or(String::new())),
            }
            .into(),
        );

        rows.push(
            StrProperty {
                name: columns[2].clone(),
                property_guid: None,
                duplication_index: 9,
                value: Some(mod_data.description.clone().unwrap_or(String::new())),
            }
            .into(),
        );

        rows.push(
            StrProperty {
                name: columns[3].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.mod_version.clone()),
            }
            .into(),
        );

        rows.push(
            StrProperty {
                name: columns[4].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.game_build.clone().unwrap_or(String::new())),
            }
            .into(),
        );

        rows.push(
            ByteProperty {
                name: columns[5].clone(),
                property_guid: None,
                duplication_index: 0,
                enum_type: Some(asset.add_name_reference(String::from("SyncMode"), false) as i64),
                byte_type: unreal_asset::properties::int_property::ByteType::Long,
                value: asset.add_name_reference(String::from(coded_sync_mode), false) as i64,
            }
            .into(),
        );

        rows.push(
            StrProperty {
                name: columns[6].clone(),
                property_guid: None,
                duplication_index: 0,
                value: Some(mod_data.homepage.clone().unwrap_or(String::new())),
            }
            .into(),
        );

        rows.push(
            BoolProperty {
                name: columns[7].clone(),
                property_guid: None,
                duplication_index: 0,
                value: true, // optional modids?
            }
            .into(),
        );

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
    if asset.exports.len() != 2 {
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

    match asset.exports[1] {
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
    E: 'static + std::error::Error,
    C: IntegratorConfig<'data, T, E>,
>(
    integrator_config: &C,
    paks_path: &str,
    install_path: &str,
    refuse_mismatched_connections: bool
) -> Result<(), Error> {
    let mods_dir = fs::read_dir(paks_path)?;
    let mod_files: Vec<DirEntry> = mods_dir
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .into_string()
                .map(|e| e.ends_with("_P.pak") && e != "999-Mods_P.pak")
                .unwrap_or(false)
        })
        .collect();

    let game_dir = fs::read_dir(install_path)?;
    let game_files: Vec<File> = game_dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|e| e == "pak").unwrap_or(false))
        .filter_map(|e| File::open(&e.path()).ok())
        .collect();
    if game_files.len() == 0 {
        return Err(IntegrationError::game_not_found().into());
    }

    let mut mods = Vec::new();
    let mut optional_mods_data = Vec::new();
    for mod_file in &mod_files {
        let stream = File::open(&mod_file.path())?;
        let mut pak = PakFile::new(&stream);
        pak.load_records()?;

        let record = &pak.read_record(&String::from("metadata.json"))?;
        let metadata: Metadata = serde_json::from_slice(&record)?;
        mods.push(metadata.clone());

        let optional_metadata: Value = serde_json::from_slice(&record)?;
        optional_mods_data.push(optional_metadata);
    }

    if mods.len() > 0 {
        let path = Path::new(install_path).join("999-Mods_P.pak");
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        let file = OpenOptions::new().append(true).open(&path)?;
        let mut generated_pak = PakFile::new(&file);
        generated_pak.init_empty(8)?;

        let mut list_of_mods = read_in_memory(
            LIST_OF_MODS_ASSET.to_vec(),
            None,
            integrator_config.get_engine_version(),
        )?;
        bake_mod_data(&mut list_of_mods, &mods)?;
        write_asset(
            &mut generated_pak,
            &list_of_mods,
            &(integrator_config.get_game_name() + "/Content/Integrator/ListOfMods.uasset"),
        )?;

        #[cfg(not(bulk_data))]
        let integrator_statics_bulk = None;
        #[cfg(bulk_data)]
        let integrator_statics_bulk = Some(INTEGRATOR_STATICS_BULK.to_vec());

        let mut integrator_statics = read_in_memory(
            INTEGRATOR_STATICS_ASSET.to_vec(),
            integrator_statics_bulk,
            integrator_config.get_engine_version(),
        )?;

        bake_integrator_data(
            &mut integrator_statics,
            integrator_config.get_integrator_version(),
            refuse_mismatched_connections,
        )?;
        write_asset(
            &mut generated_pak,
            &integrator_statics,
            &(integrator_config.get_game_name()
                + "/Content/Integrator/IntegratorStatics_BP.uasset"),
        )?;

        generated_pak.write_record(
            &String::from("metadata.json"),
            &METADATA_JSON.to_vec(),
            &unreal_pak::CompressionMethod::Zlib,
        )?;

        for entry in &COPY_OVER {
            generated_pak.write_record(
                &(integrator_config.get_game_name() + "/Content/Integrator/" + entry.1),
                &entry.0.to_vec(),
                &unreal_pak::CompressionMethod::Zlib,
            )?;
        }

        let mut game_paks = Vec::new();
        for game_file in &game_files {
            let pak = PakFile::new(&game_file);
            game_paks.push(pak);
        }

        for (name, mut exec) in integrator_config.get_handlers() {
            let all_mods: Vec<&Value> = optional_mods_data
                .iter()
                .filter(|e| e[name.clone()] != Value::Null)
                .collect();
            exec(
                integrator_config.get_data(),
                &mut generated_pak,
                &mut game_paks,
                all_mods,
            )
            .map_err(|e| Error::other(Box::new(e)))?;
        }

        generated_pak.write_index_and_footer()?;
        file.sync_data()?;
    }

    Ok(())
}
