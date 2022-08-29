use std::collections::BTreeMap;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::Error;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicI32;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use config::InstallManager;
use directories::BaseDirs;
use eframe::egui;
use game_mod::GameModVersion;
use log::warn;
use log::{debug, error};
use mod_processing::index_file::download_index_file;
use mod_processing::index_file::IndexFileModVersion;
use parking_lot::Mutex;

use semver::Version;
use unreal_modintegrator::FileMod;
use unreal_modintegrator::IntegratorModInfo;
use unreal_modintegrator::{integrate_mods, IntegratorConfig, INTEGRATOR_PAK_FILE_NAME};

mod app;
pub mod config;
pub mod error;
pub(crate) mod game_mod;
pub mod game_path_helpers;
pub mod game_platform_managers;
mod mod_config;
mod mod_processing;
pub mod update_info;
pub mod version;

use error::{ModLoaderError, ModLoaderWarning};
use game_mod::GameMod;
use mod_config::{load_config, write_config};
use mod_processing::process_modfiles;
use unreal_modmetadata::Metadata;
use unreal_pak::PakFile;
use update_info::UpdateInfo;
use version::GameBuild;

pub use unreal_asset;
pub use unreal_modintegrator;
pub use unreal_modmetadata;
pub use unreal_pak;

use crate::mod_processing::dependencies::DependencyGraph;
use crate::mod_processing::dependencies::ModWithDependencies;

#[derive(Debug)]
pub(crate) struct ModLoaderAppData {
    /// %LocalAppData%\[GameName]\Saved\Mods
    pub mods_path: Option<PathBuf>,
    /// %LocalAppData%\[GameName]\Saved\Paks
    pub paks_path: Option<PathBuf>,
    /// game install path
    pub game_install_path: Option<PathBuf>,

    pub game_build: Option<GameBuild>,
    pub refuse_mismatched_connections: bool,
    pub files_to_process: Vec<PathBuf>,

    pub game_mods: BTreeMap<String, GameMod>,

    pub error: Option<ModLoaderError>,
    pub warnings: Vec<ModLoaderWarning>,

    pub failed: bool,

    pub dependency_graph: Option<DependencyGraph>,

    /// install managers
    pub(crate) install_managers: BTreeMap<&'static str, Box<dyn InstallManager>>,
    pub(crate) selected_game_platform: Option<String>,
}

impl ModLoaderAppData {
    pub fn set_game_platform(&mut self, platform: &str) -> bool {
        let manager = self.install_managers.get(platform);
        if let Some(manager) = manager {
            self.game_install_path = manager.get_game_install_path();
            self.game_build = manager.get_game_build();
            self.paks_path = manager.get_paks_path();

            self.selected_game_platform = Some(platform.to_string());

            write_config(self);
            return true;
        }
        false
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_install_manager(&self) -> Option<&Box<dyn InstallManager>> {
        if let Some(platform) = &self.selected_game_platform {
            return self.install_managers.get(&platform.as_str());
        }

        None
    }
}

struct BackgroundThreadData {
    pub(crate) data: Arc<Mutex<ModLoaderAppData>>,
    pub(crate) should_exit: Arc<AtomicBool>,
    pub(crate) ready_exit: Arc<AtomicBool>,
    pub(crate) should_integrate: Arc<AtomicBool>,
    pub(crate) last_integration_time: Arc<Mutex<Instant>>,
    pub(crate) working: Arc<AtomicBool>,

    pub(crate) newer_update: Arc<Mutex<Option<UpdateInfo>>>,
    pub(crate) should_update: Arc<AtomicBool>,
    pub(crate) update_progress: Arc<AtomicI32>,
}

fn download_mod(
    mods_path: &Path,
    mod_version: &IndexFileModVersion,
) -> Result<(Metadata, PathBuf), ModLoaderWarning> {
    // this is safe because the filename has already been validated
    let file_path = mods_path.join(mod_version.file_name.clone());
    let mut file = fs::File::create(&file_path)?;

    let mut response = reqwest::blocking::get(mod_version.download_url.as_str())
        .map_err(|_| ModLoaderWarning::download_failed(mod_version.file_name.clone()))?;
    io::copy(&mut response, &mut file)?;

    drop(file);
    let file = fs::File::open(&file_path)?;

    let mut pak = PakFile::reader(&file);
    pak.load_records()?;

    let metadata = pak.get_record(&String::from("metadata.json"))?;
    let metadata = unreal_modmetadata::from_slice(metadata.data.as_ref().unwrap())
        .map_err(|_| ModLoaderWarning::invalid_metadata(mod_version.file_name.clone()))?;

    Ok((metadata, file_path))
}

fn download_mods(
    mods_path: &Path,
    files_to_download: &Vec<GameModVersion>,
) -> Vec<ModLoaderWarning> {
    // let mut resolved = HashMap::new();
    let mut warnings = Vec::new();

    if !files_to_download.is_empty() {
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
    }

    warnings
}

fn background_work<'a, C, D, T: 'a, E: 'static + std::error::Error + Send>(
    config: C,
    background_thread_data: BackgroundThreadData,
) -> Result<(), Error>
where
    D: 'static + IntegratorConfig<'a, T, E>,
    C: 'static + config::GameConfig<'a, D, T, E>,
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
            .join(D::GAME_NAME)
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

        let mod_files: Vec<PathBuf> = mods_dir
            .filter_map(|e| e.ok())
            .filter(|e| match e.file_name().into_string() {
                Ok(s) => s.ends_with("_P.pak") && s != INTEGRATOR_PAK_FILE_NAME,
                Err(_) => false,
            })
            .map(|e| e.path())
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
    match startup_work() {
        Ok(_) => {}
        Err(err) => {
            background_thread_data.data.lock().error = Some(err);
        }
    }

    debug!(
        "Background thread startup took {} milliseconds",
        start.elapsed().as_millis()
    );

    // background loop
    loop {
        if background_thread_data.should_exit.load(Ordering::Acquire) {
            debug!("Background thread exiting...");
            background_thread_data
                .ready_exit
                .store(true, Ordering::Release);
            break;
        }

        // auto update
        let newer_update = background_thread_data.newer_update.lock();
        if newer_update.is_some() {
            drop(newer_update);
            if background_thread_data.should_update.load(Ordering::Acquire) {
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
                background_thread_data
                    .should_exit
                    .store(true, Ordering::Release);
                continue;
            }
            thread::sleep(Duration::from_millis(50));
            continue;
        } else {
            drop(newer_update);
        }

        let mods_path = background_thread_data
            .data
            .lock()
            .mods_path
            .clone()
            .unwrap();

        // process dropped files
        let mut data_guard = background_thread_data.data.lock();
        if !data_guard.files_to_process.is_empty() {
            background_thread_data
                .working
                .store(true, Ordering::Release);

            let files_to_process = data_guard
                .files_to_process
                .clone()
                .iter()
                .filter_map(|file_path| {
                    let file_name = file_path.file_name().unwrap();

                    // copy the file to the mods directory
                    let new_file_path = mods_path.join(file_name);
                    match fs::copy(file_path, &new_file_path) {
                        Ok(_) => Some(new_file_path),
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
                .collect::<Vec<PathBuf>>();
            data_guard.files_to_process.clear();

            // drop here because process_modfiles takes time
            drop(data_guard);

            let warnings = process_modfiles(&files_to_process, &background_thread_data.data, true);
            debug!("warnings: {:?}", warnings);
            background_thread_data.data.lock().warnings.extend(warnings);

            background_thread_data
                .should_integrate
                .store(true, Ordering::Release);
        } else {
            drop(data_guard);
        }

        let mut data_guard = background_thread_data.data.lock();

        // mod deletion
        let mut to_remove = Vec::new();
        let mut del_warnings = Vec::new();
        for (mod_id, game_mod) in data_guard.game_mods.iter_mut().filter(|(_, m)| m.remove) {
            // remove file for each version
            for (_, version) in game_mod.versions.iter_mut().filter(|v| v.1.downloaded) {
                println!("Removing {:?}", mods_path.join(&version.file_name));
                match fs::remove_file(mods_path.join(&version.file_name)) {
                    Ok(_) => {}
                    Err(err) => {
                        del_warnings.push(ModLoaderWarning::io_error_with_message(
                            "Removing file from mods directory".to_owned(),
                            err,
                        ));
                    }
                }
            }
            to_remove.push(mod_id.clone());
        }
        for mod_id in to_remove {
            data_guard.game_mods.remove(&mod_id);
        }
        data_guard.warnings.extend(del_warnings);

        // integrate
        if background_thread_data
            .should_integrate
            .load(Ordering::Acquire)
            && data_guard.game_install_path.is_some()
            && data_guard.warnings.is_empty()
        {
            data_guard.failed = false;
            let integration_work = (|| -> Result<(), ModLoaderWarning> {
                background_thread_data
                    .should_integrate
                    .store(false, Ordering::Release);
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
                        .map(|f| mods_path.join(f.file_name.clone()))
                        .collect::<Vec<_>>(),
                    &background_thread_data.data,
                    false,
                );
                debug!("warnings: {:?}", warnings);
                background_thread_data.data.lock().warnings.extend(warnings);

                // fetch dependencies

                let mut download_pool = HashMap::new();
                let mut first_round = Vec::new();

                for (mod_id, enabled_mod) in background_thread_data
                    .data
                    .lock()
                    .game_mods
                    .iter()
                    .filter(|(_, game_mod)| game_mod.enabled)
                {
                    for (dependency_mod_id, dependency) in &enabled_mod.dependencies {
                        if let Some(download_info) = dependency.download.as_ref() {
                            let entry = download_pool
                                .entry(dependency_mod_id.clone())
                                .or_insert_with(Vec::new);
                            if !entry.contains(download_info) {
                                entry.push(download_info.clone());
                            }
                        }
                    }

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
                            let (_, index_file) = download_index_file(mod_id.clone(), download)?;

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
                            let (metadata, mod_path) = download_mod(&mods_path, available_version)?;
                            mods_to_install.push(GameModVersion {
                                mod_id: metadata.mod_id.clone(),
                                file_name: available_version.file_name.clone(),
                                downloaded: true,
                                download_url: Some(available_version.download_url.clone()),
                                metadata: Some(metadata),
                            });
                            to_enable.push(mod_id.clone());
                            downloaded_mods.push(mod_path);
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

                background_thread_data.data.lock().dependency_graph = Some(graph);

                if !warnings.is_empty() {
                    background_thread_data.data.lock().failed = true;
                    background_thread_data.data.lock().warnings.extend(warnings);
                    return Ok(());
                }

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
                drop(data_guard);

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

                // copy new files
                for mod_version in mods_to_install {
                    fs::copy(
                        mods_path.join(mod_version.file_name.as_str()),
                        paks_path.join(mod_version.file_name.as_str()),
                    )
                    .map(|_| ())?;

                    mods_to_integrate.push(
                        FileMod {
                            path: paks_path.join(mod_version.file_name.as_str()),
                            mod_id: mod_version.mod_id.clone(),
                        }
                        .into(),
                    );
                }

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
                    &install_path.join(D::GAME_NAME).join("Content").join("Paks"),
                    refuse_mismatched_connections,
                )?;

                debug!(
                    "Integration took {} milliseconds",
                    start_integrator.elapsed().as_millis()
                );

                *background_thread_data.last_integration_time.lock() = Instant::now();

                // update config file
                write_config(&mut background_thread_data.data.lock());

                Ok(())
            })();
            match integration_work {
                Ok(_) => {}
                Err(err) => {
                    warn!("Integration work error: {:?}", err);
                    background_thread_data.data.lock().warnings.push(err);
                }
            }
        } else {
            drop(data_guard);
        }

        background_thread_data
            .working
            .store(false, Ordering::Release);
        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

pub fn run<'a, C, D, T: 'a, E: 'static + std::error::Error + Send>(config: C)
where
    D: 'static + IntegratorConfig<'a, T, E>,
    C: 'static + config::GameConfig<'a, D, T, E>,
{
    let data = Arc::new(Mutex::new(ModLoaderAppData {
        mods_path: None,
        paks_path: None,
        game_install_path: None,

        game_build: None,
        refuse_mismatched_connections: true,
        game_mods: BTreeMap::new(),
        files_to_process: Vec::new(),

        error: None,
        warnings: Vec::new(),
        install_managers: config.get_install_managers(),
        selected_game_platform: None,
        failed: false,
        dependency_graph: None,
    }));

    let icon_data = config.get_icon();

    let should_exit = Arc::new(AtomicBool::new(false));
    let ready_exit = Arc::new(AtomicBool::new(false));
    let should_integrate = Arc::new(AtomicBool::new(true));
    let last_integration_time = Arc::new(Mutex::new(Instant::now()));
    let working = Arc::new(AtomicBool::new(true));

    let newer_update = Arc::new(Mutex::new(config.get_newer_update().ok().flatten()));

    let should_update = Arc::new(AtomicBool::new(false));
    let update_progress = Arc::new(AtomicI32::new(0));

    // instantiate the GUI app
    let app = app::ModLoaderApp {
        data: Arc::clone(&data),
        window_title: C::WINDOW_TITLE.to_owned(),

        should_exit: Arc::clone(&should_exit),
        ready_exit: Arc::clone(&ready_exit),

        should_integrate: Arc::clone(&should_integrate),
        last_integration_time: Arc::clone(&last_integration_time),

        working: Arc::clone(&working),

        platform_selector_open: false,
        selected_mod_id: None,
        newer_update: Arc::clone(&newer_update),
        should_update: Arc::clone(&should_update),
        update_progress: Arc::clone(&update_progress),

        modloader_version: C::CRATE_VERSION,
    };

    // spawn a background thread to handle long running tasks

    let background_thread_data = BackgroundThreadData {
        data,
        should_exit,
        ready_exit,
        should_integrate,
        last_integration_time,
        working,
        newer_update,
        should_update,
        update_progress,
    };

    thread::Builder::new()
        .name("background".to_string())
        .spawn(move || background_work(config, background_thread_data))
        .unwrap_or_else(|_| {
            error!("Failed to start background thread");
            panic!();
        });

    let icon_data = match icon_data {
        Some(data) => Some(eframe::IconData {
            rgba: data.data.to_vec(),
            width: data.width,
            height: data.height,
        }),
        None => None,
    };

    // run the GUI app
    eframe::run_native(
        app.window_title.clone().as_str(),
        eframe::NativeOptions {
            initial_window_size: Some(eframe::egui::vec2(623.0, 600.0)),
            icon_data,
            ..eframe::NativeOptions::default()
        },
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.iter_mut().for_each(|font| {
                font.1.tweak.scale = 1.15;
            });
            cc.egui_ctx.set_fonts(fonts);

            cc.egui_ctx.set_style(egui::Style::default());

            Box::new(app)
        }),
    );
}
