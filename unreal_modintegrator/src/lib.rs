use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use error::IntegrationError;
use log::debug;
use serde_json::Value;

use unreal_asset::engine_version::EngineVersion;
use unreal_asset::properties::int_property::BytePropertyValue;
use unreal_asset::{
    exports::{data_table_export::DataTable, Export, ExportBaseTrait},
    properties::{
        int_property::{BoolProperty, ByteProperty},
        str_property::StrProperty,
        struct_property::StructProperty,
        Property, PropertyDataTrait,
    },
    types::FName,
    Asset,
};
use unreal_modmetadata::{Metadata, SyncMode};
use unreal_pak::{pakversion::PakVersion, PakMemory, PakReader};

mod assets;
pub mod error;
mod handlers;
pub mod helpers;
pub mod macros;

use assets::{COPY_OVER, INTEGRATOR_STATICS_ASSET, LIST_OF_MODS_ASSET, METADATA_JSON};
#[cfg(not(feature = "no_bulk_data"))]
use assets::{INTEGRATOR_STATICS_BULK, LIST_OF_MODS_BULK};

pub use crate::error::Error;
use crate::handlers::handle_persistent_actors;
use crate::helpers::write_asset;

pub trait IntegratorInfo {}

pub const INTEGRATOR_PAK_FILE_NAME: &str = "900-ModIntegrator_P.pak";

pub enum IntegratorMod<E: std::error::Error> {
    File(FileMod),
    Baked(BakedMod),
    Dynamic(Box<dyn DynamicMod<E>>),
}

pub trait IntegratorModInfo {
    fn get_mod_id(&self) -> String;
    fn get_priority(&self) -> u32;
    fn is_core(&self) -> bool;
}

impl<E: std::error::Error> IntegratorModInfo for IntegratorMod<E> {
    fn get_mod_id(&self) -> String {
        match self {
            IntegratorMod::File(file_mod) => file_mod.get_mod_id(),
            IntegratorMod::Baked(baked_mod) => baked_mod.get_mod_id(),
            IntegratorMod::Dynamic(dynamic_mod) => dynamic_mod.get_mod_id(),
        }
    }

    fn is_core(&self) -> bool {
        match self {
            IntegratorMod::File(file_mod) => file_mod.is_core(),
            IntegratorMod::Baked(baked_mod) => baked_mod.is_core(),
            IntegratorMod::Dynamic(dynamic_mod) => dynamic_mod.is_core(),
        }
    }

    fn get_priority(&self) -> u32 {
        match self {
            IntegratorMod::File(file_mod) => file_mod.get_priority(),
            IntegratorMod::Baked(baked_mod) => baked_mod.get_priority(),
            IntegratorMod::Dynamic(dynamic_mod) => dynamic_mod.get_priority(),
        }
    }
}

impl<E: std::error::Error> From<BakedMod> for IntegratorMod<E> {
    fn from(e: BakedMod) -> Self {
        IntegratorMod::Baked(e)
    }
}

impl<E: std::error::Error> From<FileMod> for IntegratorMod<E> {
    fn from(e: FileMod) -> Self {
        IntegratorMod::File(e)
    }
}

pub struct FileMod {
    pub path: PathBuf,
    pub mod_id: String,
    pub priority: u32,
}

impl IntegratorModInfo for FileMod {
    fn get_mod_id(&self) -> String {
        self.mod_id.clone()
    }

    fn is_core(&self) -> bool {
        false
    }

    fn get_priority(&self) -> u32 {
        self.priority
    }
}

pub struct BakedMod {
    pub data: &'static [u8],
    pub mod_id: String,
    pub filename: &'static str,
    pub priority: u32,
    pub is_core: bool,
}

impl BakedMod {
    pub fn write(&self, path: &Path) -> Result<File, Error> {
        let mut file = File::create(path.join(self.filename))?;
        file.write_all(self.data)?;
        drop(file);

        let file = File::open(path.join(self.filename))?;
        Ok(file)
    }
}

impl IntegratorModInfo for BakedMod {
    fn get_mod_id(&self) -> String {
        self.mod_id.clone()
    }

    fn is_core(&self) -> bool {
        self.is_core
    }

    fn get_priority(&self) -> u32 {
        self.priority
    }
}

pub trait DynamicMod<E: std::error::Error>: IntegratorModInfo {
    fn integrate(
        &self,
        integrated_pak: &mut PakMemory,
        game_paks: &mut Vec<PakReader<File>>,
        mod_paks: &mut Vec<PakReader<File>>,
    ) -> Result<(), E>;
}

pub type HandlerFn<D, E> = dyn FnMut(
    &D,
    &mut PakMemory,
    &mut Vec<PakReader<File>>,
    &mut Vec<PakReader<File>>,
    &Vec<Value>,
) -> Result<(), E>;

pub trait IntegratorConfig<'data, D, E: std::error::Error + 'static> {
    fn get_data(&self) -> &'data D;
    fn get_handlers(&self) -> HashMap<String, Box<HandlerFn<D, E>>>;

    fn get_baked_mods(&self) -> Vec<IntegratorMod<E>>;

    const GAME_NAME: &'static str;
    const INTEGRATOR_VERSION: &'static str;
    const ENGINE_VERSION: EngineVersion;
}

fn read_in_memory(
    uasset: Vec<u8>,
    uexp: Option<Vec<u8>>,
    engine_version: EngineVersion,
) -> Result<Asset<Cursor<Vec<u8>>>, Error> {
    let mut asset = Asset::new(Cursor::new(uasset), uexp.map(Cursor::new));
    asset.set_engine_version(engine_version);
    asset.parse_data()?;
    Ok(asset)
}

fn bake_mod_data(asset: &mut Asset<Cursor<Vec<u8>>>, mods: &Vec<Metadata>) -> Result<(), Error> {
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
        asset.add_fname(coded_sync_mode);

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
                enum_type: Some(FName::from_slice("SyncMode")),
                value: BytePropertyValue::FName(FName::from_slice(coded_sync_mode)),
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
    asset: &mut Asset<Cursor<Vec<u8>>>,
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
    mods: &[IntegratorMod<E>],
    paks_path: &Path,
    game_path: &Path,
    refuse_mismatched_connections: bool,
) -> Result<(), Error> {
    debug!(
        "Integrating {} mods, refuse_mismatched_connections: {}",
        mods.len(),
        refuse_mismatched_connections
    );

    let baked_mods = integrator_config.get_baked_mods();
    let core_mods = baked_mods.iter().filter(|e| e.is_core());

    let enabled_baked_mods = baked_mods.iter().filter(|e| !e.is_core()).filter(|e| {
        mods.iter()
            .any(|provided_mod| provided_mod.get_mod_id() == e.get_mod_id())
    });

    let game_dir = fs::read_dir(game_path)?;
    let game_files: Vec<File> = game_dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|e| e == "pak").unwrap_or(false))
        .filter_map(|e| File::open(e.path()).ok())
        .collect();
    if game_files.is_empty() {
        return Err(IntegrationError::game_not_found().into());
    }

    let mod_files = mods
        .iter()
        .chain(core_mods.into_iter())
        .chain(enabled_baked_mods.into_iter())
        .filter_map(|e| match e {
            IntegratorMod::File(file_mod) => File::open(&file_mod.path).ok(),
            IntegratorMod::Baked(baked_mod) => baked_mod.write(paks_path).ok(),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut mod_paks = Vec::new();
    let mut read_mods = Vec::new();
    let mut optional_mods_data = HashMap::new();

    for mod_file in &mod_files {
        let mut pak = PakReader::new(mod_file);
        pak.load_index()?;

        let record = pak.read_entry(&String::from("metadata.json"))?;
        let metadata = unreal_modmetadata::from_slice(&record)?;
        read_mods.push(metadata.clone());

        debug!(
            "Integrating modid {} version {}",
            metadata.mod_id, metadata.mod_version
        );

        for (name, data) in &metadata.integrator {
            optional_mods_data
                .entry(name.clone())
                .or_insert_with(Vec::new)
                .push(data.clone());
        }

        mod_paks.push(pak);
    }

    if !mods.is_empty() {
        let mut generated_pak = PakMemory::new(PakVersion::FnameBasedCompressionMethod);

        #[cfg(not(feature = "no_bulk_data"))]
        let list_of_mods_bulk = Some(LIST_OF_MODS_BULK.to_vec());
        #[cfg(feature = "no_bulk_data")]
        let list_of_mods_bulk = None;

        let mut list_of_mods = read_in_memory(
            LIST_OF_MODS_ASSET.to_vec(),
            list_of_mods_bulk,
            C::ENGINE_VERSION,
        )?;
        bake_mod_data(&mut list_of_mods, &read_mods)?;
        write_asset(
            &mut generated_pak,
            &list_of_mods,
            &(C::GAME_NAME.to_owned() + "/Content/Integrator/ListOfMods.uasset"),
        )?;

        #[cfg(not(feature = "no_bulk_data"))]
        let integrator_statics_bulk = Some(INTEGRATOR_STATICS_BULK.to_vec());
        #[cfg(feature = "no_bulk_data")]
        let integrator_statics_bulk = None;

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

        generated_pak.set_entry(String::from("metadata.json"), METADATA_JSON.to_vec());

        for entry in &COPY_OVER {
            generated_pak.set_entry(
                C::GAME_NAME.to_owned() + "/Content/Integrator/" + entry.1,
                entry.0.to_vec(),
            );
        }

        let mut game_paks = Vec::new();
        for game_file in &game_files {
            let mut pak = PakReader::new(game_file);
            pak.load_index()?;
            game_paks.push(pak);
        }

        let empty_vec: Vec<Value> = Vec::new();

        let persistent_actor_maps: Vec<&str> = optional_mods_data
            .get("persistent_actor_maps")
            .unwrap_or(&empty_vec)
            .iter()
            .filter_map(|e| e.as_array())
            .flat_map(|e| e.iter().filter_map(|e| e.as_str()))
            .collect();

        let persistent_actors = optional_mods_data
            .get("persistent_actors")
            .unwrap_or(&empty_vec);

        handle_persistent_actors(
            C::GAME_NAME,
            &persistent_actor_maps,
            &mut generated_pak,
            &mut game_paks,
            &mut mod_paks,
            persistent_actors,
        )?;

        for dynamic_mod in mods.iter() {
            if let IntegratorMod::Dynamic(dynamic_mod) = dynamic_mod {
                dynamic_mod
                    .integrate(&mut generated_pak, &mut game_paks, &mut mod_paks)
                    .map_err(|e| Error::other(Box::new(e)))?;
            }
        }

        for (name, mut exec) in integrator_config.get_handlers() {
            let all_mods = optional_mods_data.get(&name).unwrap_or(&empty_vec);

            exec(
                integrator_config.get_data(),
                &mut generated_pak,
                &mut game_paks,
                &mut mod_paks,
                all_mods,
            )
            .map_err(|e| Error::other(Box::new(e)))?;
        }

        let path = Path::new(paks_path).join(INTEGRATOR_PAK_FILE_NAME);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        generated_pak.write(&mut file)?;
        file.sync_data()?;
    }

    Ok(())
}
