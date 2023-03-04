use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Error};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    Arc,
};
use std::time::Instant;

use directories::BaseDirs;
use log::{debug, warn};
use parking_lot::Mutex;
use semver::Version;
use sha2::{Digest, Sha256};

use unreal_modintegrator::{
    integrate_mods, FileMod, IntegratorConfig, IntegratorModInfo, INTEGRATOR_PAK_FILE_NAME,
};
use unreal_modmetadata::Metadata;
use unreal_pak::PakReader;

use crate::config;
use crate::error::{ModLoaderError, ModLoaderWarning};
use crate::game_mod::{self, GameModVersion};
use crate::mod_config::{load_config, write_config};
use crate::mod_processing::{
    dependencies::{DependencyGraph, ModWithDependencies},
    index_file::{download_index_file, IndexFileModVersion},
    process_modfiles,
};
use crate::update_info::UpdateInfo;
use crate::FileToProcess;
use crate::ModLoaderAppData;
use crate::UntrustedMod;

pub(crate) struct BackgroundThreadData {
    pub(crate) data: Arc<Mutex<ModLoaderAppData>>,
    pub(crate) ready_exit: Arc<AtomicBool>,
    pub(crate) last_integration_time: Arc<Mutex<Instant>>,
    pub(crate) working: Arc<AtomicBool>,

    pub(crate) newer_update: Arc<Mutex<Option<UpdateInfo>>>,
    pub(crate) should_update: Arc<AtomicBool>,
    pub(crate) update_progress: Arc<AtomicI32>,
}

pub(crate) enum BackgroundThreadMessage {
    Import(Vec<FileToProcess>),
    RemoveMod(String),
    Integrate(Instant),
    WriteConfig,
    UpdateApp,
    LaunchGame,
    Exit,
}

impl BackgroundThreadMessage {
    pub fn integrate() -> Self {
        BackgroundThreadMessage::Integrate(Instant::now())
    }
}

fn download_mod(
    mods_path: &Path,
    mod_version: &IndexFileModVersion,
) -> Result<(Metadata, PathBuf), ModLoaderWarning> {
    // this is safe because the filename has already been validated
    let mut response = reqwest::blocking::get(mod_version.download_url.as_str())
        .map_err(|e| ModLoaderWarning::download_failed(mod_version.file_name.clone(), e))?;

    let file_path = mods_path.join(mod_version.file_name.clone());
    let mut file = fs::File::create(&file_path)?;

    io::copy(&mut response, &mut file)?;

    drop(file);
    let file = fs::File::open(&file_path)?;

    let mut pak = PakReader::new(&file);
    pak.load_index()?;

    let metadata = pak.read_entry(&String::from("metadata.json"))?;
    let metadata = unreal_modmetadata::from_slice(&metadata)
        .map_err(|_| ModLoaderWarning::invalid_metadata(mod_version.file_name.clone()))?;

    Ok((metadata, file_path))
}

fn download_mods(mods_path: &Path, files_to_download: &[GameModVersion]) -> Vec<ModLoaderWarning> {
    // let mut resolved = HashMap::new();
    let mut warnings = Vec::new();

    // ? Maybe parallelize this?
    for mod_version in files_to_download
        .iter()
        .filter(|e| e.download_url.is_some())
        .map(|e| IndexFileModVersion::new(e.download_url.clone().unwrap(), e.file_name.clone()))
    {
        if let Err(err) = download_mod(mods_path, &mod_version) {
            debug!("Failed to download {:?} {:?}", mod_version.file_name, err);
            warnings.push(err);
        }
    }

    warnings
}

pub(crate) fn background_work<'data, GC, IC, D: 'data, E: 'static + std::error::Error + Send>(
    config: GC,
    background_thread_data: BackgroundThreadData,
    receiver: Receiver<BackgroundThreadMessage>,
) -> Result<(), Error>
where
    IC: 'static + IntegratorConfig<'data, D, E>,
    GC: 'static + config::GameConfig<'data, IC, D, E>,
{
    debug!("Starting background thread");

    let start = Instant::now();
    background_thread_data
        .working
        .store(true, Ordering::Release);

    let startup_work = || -> Result<(), ModLoaderError> {
        let mods_path = BaseDirs::new()
            .ok_or_else(ModLoaderError::no_base_path)?
            .data_local_dir()
            .join(IC::GAME_NAME)
            .join("Saved")
            .join("Mods");

        background_thread_data.data.lock().mods_path = Some(mods_path.clone());

        // ensure the base_path/Mods directory exists
        fs::create_dir_all(&mods_path).map_err(|err| {
            ModLoaderError::io_error_with_message("create Mods directory".to_owned(), err)
        })?;

        // gather mods
        let mods_dir = fs::read_dir(&mods_path).map_err(|err| {
            ModLoaderError::io_error_with_message("read Mods directory".to_owned(), err)
        })?;

        let mod_files = mods_dir
            .filter_map(|e| e.ok())
            .filter(|e| match e.file_name().into_string() {
                Ok(s) => s.ends_with("_P.pak") && s != INTEGRATOR_PAK_FILE_NAME,
                Err(_) => false,
            })
            .map(|e| FileToProcess::new(e.path(), false))
            .collect();

        let warnings = process_modfiles(&mod_files, &background_thread_data.data, false);
        debug!("warnings: {:?}", warnings);

        let mut data_guard = background_thread_data.data.lock();
        data_guard.warnings.extend(warnings);

        // load config
        load_config(&mut data_guard);

        // debug!("{:#?}", data_guard.game_mods);
        Ok(())
    };
    if let Err(err) = startup_work() {
        warn!("Startup work error: {}", err);
        background_thread_data.data.lock().error = Some(err);
    }

    background_thread_data
        .working
        .store(false, Ordering::Release);

    debug!(
        "Background thread startup took {} milliseconds",
        start.elapsed().as_millis()
    );

    let mods_path = background_thread_data
        .data
        .lock()
        .mods_path
        .clone()
        .unwrap();

    let mut last_integration_time = None;

    while let Ok(message) = receiver.recv() {
        match message {
            BackgroundThreadMessage::Import(files_to_process) => {
                background_thread_data
                    .working
                    .store(true, Ordering::Release);

                let mut data_guard = background_thread_data.data.lock();
                let files_to_process = files_to_process
                    .clone()
                    .iter()
                    .filter_map(|file| {
                        let file_path = &file.path;
                        let file_name = file_path.file_name().unwrap();

                        // copy the file to the mods directory
                        let new_file_path = mods_path.join(file_name);
                        match fs::copy(file_path, &new_file_path) {
                            Ok(_) => Some(FileToProcess::new(new_file_path, file.newly_added)),
                            Err(err) => {
                                data_guard
                                    .warnings
                                    .push(ModLoaderWarning::io_error_with_message(
                                        "Copying file to mods directory".to_owned(),
                                        err,
                                    ));
                                None
                            }
                        }
                    })
                    .collect::<Vec<FileToProcess>>();
                data_guard.files_to_process.clear();

                // drop here because process_modfiles takes time
                drop(data_guard);

                let warnings =
                    process_modfiles(&files_to_process, &background_thread_data.data, true);
                debug!("warnings: {:?}", warnings);
                background_thread_data.data.lock().warnings.extend(warnings);
            }
            BackgroundThreadMessage::RemoveMod(mod_id) => {
                let mut data_guard = background_thread_data.data.lock();
                let mut deletion_warnings = Vec::new();

                if let Some(game_mod) = data_guard.game_mods.get(&mod_id) {
                    // remove file for each version
                    for (_, version) in game_mod.versions.iter().filter(|v| v.1.downloaded) {
                        debug!("Removing {:?}", mods_path.join(&version.file_name));
                        match fs::remove_file(mods_path.join(&version.file_name)) {
                            Ok(_) => {}
                            Err(err) => {
                                deletion_warnings.push(ModLoaderWarning::io_error_with_message(
                                    "Removing file from mods directory".to_owned(),
                                    err,
                                ));
                            }
                        }
                    }
                }

                data_guard.game_mods.remove(&mod_id);
                data_guard.warnings.extend(deletion_warnings);
            }
            BackgroundThreadMessage::UpdateApp => {
                let newer_update = background_thread_data.newer_update.lock();
                if newer_update.is_some() {
                    drop(newer_update);
                    let update_progress = Arc::clone(&background_thread_data.update_progress);
                    config
                        .update_modloader(Box::new(move |progress| {
                            update_progress
                                .store(f32::round(progress * 100.0) as i32, Ordering::Release);
                        }))
                        .unwrap();
                    background_thread_data
                        .should_update
                        .store(false, Ordering::Release);

                    Command::new(env::current_exe().unwrap()).spawn().unwrap();
                    break;
                }
            }
            BackgroundThreadMessage::Integrate(time) => {
                if let Some(last_integration_time) = last_integration_time {
                    if time < last_integration_time {
                        continue;
                    }
                }

                last_integration_time = Some(Instant::now());

                let mut data_guard = background_thread_data.data.lock();

                if data_guard.game_install_path.is_none() {
                    continue;
                }
                data_guard.failed = false;

                // TODO this should at somepoint be changed to `-> Result<Vec<ModLoaderWarning>, ModLoaderError>`
                // to properly convey that some things might critically fail.
                let integration_work = || -> Result<(), ModLoaderWarning> {
                    background_thread_data
                        .working
                        .store(true, Ordering::Release);

                    let start_pre = Instant::now();

                    // gather mods to be installed
                    let mut mods_to_install = data_guard
                        .game_mods
                        .iter()
                        .filter(|(_, m)| m.enabled)
                        .map(|(_, m)| {
                            m.versions
                                .get(&m.selected_version.clone().unwrap())
                                .unwrap()
                                .clone()
                        })
                        .collect::<Vec<_>>();

                    let paks_path = data_guard.paks_path.as_ref().unwrap().to_owned();
                    let install_path = data_guard.game_install_path.as_ref().unwrap().to_owned();
                    let refuse_mismatched_connections = data_guard.refuse_mismatched_connections;

                    #[cfg(feature = "cpp_loader")]
                    let cpp_loader_extract_path = data_guard
                        .cpp_loader_extract_path
                        .as_ref()
                        .unwrap()
                        .to_owned();

                    drop(data_guard);
                    debug!(
                        "Mods to install: {:?}",
                        mods_to_install
                            .iter()
                            .map(|m| &m.file_name)
                            .collect::<Vec<_>>()
                    );

                    let warnings = download_mods(&mods_path, &mods_to_install);
                    background_thread_data.data.lock().warnings.extend(warnings);

                    // process newly downloaded files
                    let warnings = process_modfiles(
                        &mods_to_install
                            .iter()
                            .map(|f| FileToProcess::new(mods_path.join(f.file_name.clone()), false))
                            .collect::<Vec<_>>(),
                        &background_thread_data.data,
                        false,
                    );
                    debug!("warnings: {:?}", warnings);
                    background_thread_data.data.lock().warnings.extend(warnings);

                    // fetch dependencies

                    let mut first_round = Vec::new();

                    for (mod_id, enabled_mod) in background_thread_data
                        .data
                        .lock()
                        .game_mods
                        .iter()
                        .filter(|(_, game_mod)| game_mod.enabled)
                    {
                        let selected_version = match &enabled_mod.selected_version {
                            game_mod::SelectedVersion::Latest(version) => version,
                            game_mod::SelectedVersion::LatestIndirect(_) => {
                                let mut versions =
                                    enabled_mod.versions.keys().collect::<Vec<&Version>>();
                                versions.sort();
                                *versions.last().unwrap()
                            }
                            game_mod::SelectedVersion::Specific(version) => version,
                        };
                        first_round.push(ModWithDependencies::new(
                            mod_id.clone(),
                            Vec::from([selected_version.clone()]),
                            enabled_mod.dependencies.clone(),
                        ));
                    }

                    let mut graph = DependencyGraph::default();

                    let mut download_pool = HashMap::new();
                    let mut dependencies = graph.add_mods(&first_round);

                    let mut next_round = Vec::new();

                    loop {
                        next_round.clear();
                        for (mod_id, (version_req, downloads)) in &dependencies {
                            let entry = download_pool
                                .entry(mod_id.clone())
                                .or_insert_with(HashMap::new);

                            for download in downloads {
                                let (_, index_file) =
                                    download_index_file(mod_id.clone(), download)?;

                                for (version, index_version) in index_file.versions {
                                    entry.entry(version).or_insert(index_version);
                                }
                            }

                            let matching_version = entry
                                .iter()
                                .find(|(version, _)| version_req.matches(version));

                            if let Some((_, index_version)) = matching_version {
                                let file = download_mod(&mods_path, index_version);

                                if let Ok((metadata, _)) = file {
                                    let mod_info = ModWithDependencies::new(
                                        mod_id.clone(),
                                        entry.keys().cloned().collect(),
                                        metadata.dependencies,
                                    );
                                    next_round.push(mod_info);
                                }
                            }
                        }
                        dependencies.clear();
                        dependencies = graph.add_mods(&next_round);

                        if next_round.is_empty() {
                            break;
                        }
                    }

                    let (mut to_download, mut warnings) = graph.validate_graph();

                    for existing_mod in first_round {
                        to_download.remove(&existing_mod.mod_id);
                    }

                    let mut mods_to_integrate = Vec::new();
                    for baked_mod in config.get_integrator_config().get_baked_mods() {
                        if to_download.contains_key(&baked_mod.get_mod_id()) {
                            to_download.remove(&baked_mod.get_mod_id());
                            mods_to_integrate.push(baked_mod);
                        }
                    }

                    let mut downloaded_mods = Vec::new();
                    let mut to_enable = Vec::new();
                    for (mod_id, version) in to_download {
                        let available_versions =
                            download_pool.get(&mod_id).and_then(|e| e.get(&version));
                        match available_versions {
                            Some(available_version) => {
                                let (metadata, mod_path) =
                                    download_mod(&mods_path, available_version)?;

                                mods_to_install.push(GameModVersion {
                                    mod_id: metadata.mod_id.clone(),
                                    file_name: available_version.file_name.clone(),
                                    downloaded: true,
                                    download_url: Some(available_version.download_url.clone()),
                                    metadata: Some(metadata),
                                });

                                to_enable.push(mod_id.clone());
                                downloaded_mods.push(FileToProcess::new(mod_path, false));
                            }
                            None => {
                                let dependents = graph.find_mod_dependents_with_version(&mod_id);
                                warnings.push(ModLoaderWarning::unresolved_dependency(
                                    mod_id.clone(),
                                    dependents,
                                ))
                            }
                        }
                    }

                    let mut data_guard = background_thread_data.data.lock();
                    data_guard.dependency_graph = Some(graph);
                    data_guard.warnings.extend(warnings);
                    drop(data_guard);

                    // process dependencies
                    let warnings =
                        process_modfiles(&downloaded_mods, &background_thread_data.data, true);
                    background_thread_data.data.lock().warnings.extend(warnings);

                    let mut data_guard = background_thread_data.data.lock();
                    for to_enable in to_enable {
                        if let Some(game_mod) = data_guard.game_mods.get_mut(&to_enable) {
                            game_mod.enabled = true;
                        }
                    }

                    // remove all old files
                    match fs::remove_dir_all(&paks_path) {
                        Ok(_) => Ok(()),
                        Err(err) => match err.kind() {
                            // this is fine
                            std::io::ErrorKind::NotFound => Ok(()),
                            _ => Err(ModLoaderWarning::io_error_with_message(
                                "Removing old paks directory failed".to_owned(),
                                err,
                            )),
                        },
                    }?;
                    fs::create_dir_all(&paks_path)?;

                    let trusted_mods = data_guard.trusted_mods.clone();
                    let mut untrusted_mods = Vec::new();

                    // copy new files
                    let mods_to_install = mods_to_install
                        .into_iter()
                        .map(|e| {
                            (
                                data_guard
                                    .game_mods
                                    .get(&e.mod_id)
                                    .unwrap()
                                    .selected_version
                                    .to_string(),
                                e,
                            )
                        })
                        .collect::<Vec<_>>();

                    drop(data_guard);

                    for (version_string, mod_version) in mods_to_install {
                        let dst_path = paks_path.join(mod_version.file_name.as_str());
                        fs::copy(mods_path.join(mod_version.file_name.as_str()), &dst_path)
                            .map(|_| ())?;

                        if let Some(ref metadata) = mod_version.metadata {
                            if !metadata.cpp_loader_dlls.is_empty() {
                                let mut hasher = Sha256::new();
                                let mut file = File::open(&dst_path)?;
                                let _ = io::copy(&mut file, &mut hasher)?;
                                let hash = hasher.finalize()[..].to_vec();

                                if !trusted_mods.contains(&hash) {
                                    untrusted_mods.push(UntrustedMod::new(
                                        mod_version.mod_id.clone(),
                                        version_string,
                                        hash,
                                    ));
                                }
                            }
                        }

                        mods_to_integrate.push(
                            FileMod {
                                path: dst_path,
                                mod_id: mod_version.mod_id.clone(),
                                priority: mod_version
                                    .file_name
                                    .split('-')
                                    .next()
                                    .unwrap()
                                    .parse::<u32>()
                                    .unwrap(),
                            }
                            .into(),
                        );
                    }

                    background_thread_data.data.lock().untrusted_mods = untrusted_mods;

                    mods_to_integrate.sort_by_key(|a| a.get_priority());

                    debug!(
                        "Pre Integration took {} milliseconds",
                        start_pre.elapsed().as_millis()
                    );

                    let start_integrator = Instant::now();

                    // run integrator
                    debug!("Integrating mods");
                    integrate_mods(
                        config.get_integrator_config(),
                        &mods_to_integrate,
                        &paks_path,
                        &install_path
                            .join(IC::GAME_NAME)
                            .join("Content")
                            .join("Paks"),
                        refuse_mismatched_connections,
                    )?;

                    debug!(
                        "Integration took {} milliseconds",
                        start_integrator.elapsed().as_millis()
                    );

                    #[cfg(feature = "cpp_loader")]
                    {
                        unreal_cpp_bootstrapper::bootstrap(
                            IC::GAME_NAME,
                            &cpp_loader_extract_path,
                            &paks_path,
                        )?;
                    }

                    *background_thread_data.last_integration_time.lock() = Instant::now();

                    // update config file
                    write_config(&mut background_thread_data.data.lock());

                    Ok(())
                };
                if let Err(err) = integration_work() {
                    warn!("Integration work error: {}", err);
                    let mut data_guard = background_thread_data.data.lock();
                    data_guard.warnings.push(err);
                    data_guard.failed = true;
                }

                background_thread_data
                    .working
                    .store(false, Ordering::Release);
            }
            BackgroundThreadMessage::LaunchGame => {
                fn start(data: &mut ModLoaderAppData) -> Result<(), ModLoaderWarning> {
                    let install_manager = data.get_install_manager();
                    let Some(install_manager) = install_manager else {
                        return Err(ModLoaderWarning::other("No install manager".to_string()));
                    };

                    #[cfg(feature = "cpp_loader")]
                    {
                        let config_location = install_manager.get_config_location()?;

                        fs::create_dir_all(config_location.parent().unwrap())?;
                        let file = std::fs::File::create(&config_location)?;
                        let writer = std::io::BufWriter::new(file);

                        if let Err(e) = serde_json::to_writer(writer, &data.cpp_loader_config) {
                            let _ = fs::remove_file(config_location);
                            return Err(e.into());
                        }

                        install_manager.prepare_load()?;
                    }

                    match install_manager.launch_game() {
                        Ok(_) => {
                            #[cfg(feature = "cpp_loader")]
                            install_manager.load()?;

                            Ok(())
                        }
                        Err(warn) => Err(warn),
                    }
                }

                let mut data = background_thread_data.data.lock();
                if let Err(e) = start(&mut data) {
                    data.warnings.push(e);
                }
            }
            BackgroundThreadMessage::WriteConfig => {
                // update config file
                write_config(&mut background_thread_data.data.lock());
            }
            BackgroundThreadMessage::Exit => {
                break;
            }
        }
    }

    debug!("Background thread exiting...");
    background_thread_data
        .ready_exit
        .store(true, Ordering::Release);
    Ok(())
}
