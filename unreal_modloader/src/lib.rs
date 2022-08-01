use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use config::InstallManager;
use directories::BaseDirs;
use eframe::egui;
use log::warn;
use log::{debug, error};
use parking_lot::Mutex;

use unreal_modintegrator::{integrate_mods, IntegratorConfig, INTEGRATOR_PAK_FILE_NAME};

mod app;
pub mod config;
pub mod error;
pub(crate) mod game_mod;
pub mod game_path_helpers;
pub mod game_platform_managers;
mod mod_config;
mod mod_processing;
pub mod version;

use error::{ModLoaderError, ModLoaderWarning};
use game_mod::GameMod;
use mod_config::{load_config, write_config};
use mod_processing::process_modfiles;
use version::GameBuild;

pub use unreal_modintegrator::unreal_asset;
pub use unreal_modintegrator::unreal_modmetadata;
pub use unreal_modintegrator::unreal_pak;

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
    }));

    let should_exit = Arc::new(AtomicBool::new(false));
    let ready_exit = Arc::new(AtomicBool::new(false));
    let should_integrate = Arc::new(AtomicBool::new(true));
    let last_integration_time = Arc::new(Mutex::new(Instant::now()));
    let reloading = Arc::new(AtomicBool::new(true));
    let working = Arc::new(AtomicBool::new(true));

    // instantiate the GUI app
    let app = app::ModLoaderApp {
        data: Arc::clone(&data),
        window_title: C::WINDOW_TITLE.to_owned(),

        should_exit: Arc::clone(&should_exit),
        ready_exit: Arc::clone(&ready_exit),

        should_integrate: Arc::clone(&should_integrate),
        last_integration_time: Arc::clone(&last_integration_time),

        reloading: Arc::clone(&reloading),
        working: Arc::clone(&working),

        platform_selector_open: false,
        selected_mod_id: None,
    };

    // spawn a background thread to handle long running tasks
    thread::Builder::new()
        .name("background".to_string())
        .spawn(move || {
            debug!("Starting background thread");

            let mods_path = BaseDirs::new()
                .unwrap()
                .data_local_dir()
                .join(D::GAME_NAME)
                .join("Saved")
                .join("Mods");
            println!("{:?}", mods_path);
            fs::create_dir_all(&mods_path).unwrap();

            data.lock().mods_path = Some(mods_path);

            // background loop
            loop {
                if should_exit.load(Ordering::Acquire) {
                    debug!("Background thread exiting...");
                    ready_exit.store(true, Ordering::Release);
                    break;
                }
                // reloading
                if reloading.load(Ordering::Acquire) {
                    let start = Instant::now();
                    working.store(true, Ordering::Release);

                    let data_guard = data.lock();
                    let mods_path = data_guard.mods_path.to_owned();
                    if let Some(mods_path) = mods_path {
                        drop(data_guard);

                        let startup_work = || -> Result<(), ModLoaderError> {
                            // ensure the base_path/Mods directory exists
                            fs::create_dir_all(&mods_path).map_err(|err| {
                                ModLoaderError::io_error_with_message(
                                    "Mods directory".to_owned(),
                                    err,
                                )
                            })?;

                            // gather mods
                            let mods_dir = fs::read_dir(&mods_path).map_err(|err| {
                                ModLoaderError::io_error_with_message(
                                    "Mods directory".to_owned(),
                                    err,
                                )
                            })?;

                            let mod_files: Vec<PathBuf> = mods_dir
                                .filter_map(|e| e.ok())
                                .filter(|e| match e.file_name().into_string() {
                                    Ok(s) => s.ends_with("_P.pak") && s != INTEGRATOR_PAK_FILE_NAME,
                                    Err(_) => false,
                                })
                                .map(|e| e.path())
                                .collect();

                            let warnings = process_modfiles(&mod_files, &data, false);
                            debug!("warnings: {:?}", warnings);

                            let mut data_guard = data.lock();
                            data_guard.warnings.extend(warnings);

                            // load config
                            load_config(&mut *data_guard);

                            // debug!("{:#?}", data_guard.game_mods);
                            Ok(())
                        };
                        match startup_work() {
                            Ok(_) => {}
                            Err(err) => {
                                data.lock().error = Some(err);
                            }
                        }

                        debug!(
                            "Background thread reload took {} milliseconds",
                            start.elapsed().as_millis()
                        );
                    }
                }

                working.store(false, Ordering::Release);
                reloading.store(false, Ordering::Release);

                // process dropped files
                let mut data_guard = data.lock();
                if !data_guard.files_to_process.is_empty() {
                    let files_to_process = data_guard
                        .files_to_process
                        .clone()
                        .iter()
                        .filter_map(|file_path| {
                            let file_name = file_path.file_name().unwrap();

                            // copy the file to the mods directory
                            let new_file_path =
                                data_guard.mods_path.as_ref().unwrap().join(file_name);
                            match fs::copy(file_path, &new_file_path) {
                                Ok(_) => Some(new_file_path),
                                Err(err) => {
                                    data_guard.warnings.push(
                                        ModLoaderWarning::io_error_with_message(
                                            "Copying file to mods directory".to_owned(),
                                            err,
                                        ),
                                    );
                                    None
                                }
                            }
                        })
                        .collect::<Vec<PathBuf>>();
                    data_guard.files_to_process.clear();
                    drop(data_guard);

                    let warnings = process_modfiles(&files_to_process, &data, true);
                    debug!("warnings: {:?}", warnings);
                    data.lock().warnings.extend(warnings);

                    should_integrate.store(true, Ordering::Release);
                } else {
                    drop(data_guard);
                }

                let data_guard = data.lock();
                if should_integrate.load(Ordering::Acquire)
                    && data_guard.game_install_path.is_some()
                    && data_guard.warnings.is_empty()
                {
                    working.store(true, Ordering::Release);

                    let integration_work = (|| -> Result<(), ModLoaderWarning> {
                        should_integrate.store(false, Ordering::Release);

                        // gather mods to be installed
                        let mods_to_install = data_guard
                            .game_mods
                            .iter()
                            .filter(|(_, m)| m.enabled)
                            .map(|(_, m)| {
                                m.versions
                                    .get(&m.selected_version.unwrap())
                                    .unwrap()
                                    .clone()
                            })
                            .collect::<Vec<_>>();

                        let mods_path = data_guard.mods_path.as_ref().unwrap().to_owned();
                        let paks_path = data_guard.paks_path.as_ref().unwrap().to_owned();
                        let install_path =
                            data_guard.game_install_path.as_ref().unwrap().to_owned();
                        let refuse_mismatched_connections =
                            data_guard.refuse_mismatched_connections;
                        drop(data_guard);

                        debug!(
                            "Mods to install: {:?}",
                            mods_to_install
                                .iter()
                                .map(|m| &m.file_name)
                                .collect::<Vec<_>>()
                        );

                        // download mod versions not yet downloaded
                        let files_to_download: Vec<(String, String)> = mods_to_install
                            .iter()
                            .filter_map(|m| {
                                if !m.downloaded && m.download_url.is_some() {
                                    Some((
                                        m.file_name.clone(),
                                        m.download_url.as_ref().unwrap().clone(),
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        if !files_to_download.is_empty() {
                            // ? Maybe parallelize this?
                            for (file_name, url) in &files_to_download {
                                let download = (|| -> Result<(), ModLoaderWarning> {
                                    debug!("Downloading {:?}", file_name);

                                    // this is safe because the filename has already been validated
                                    let file_path = mods_path.clone().join(file_name.clone());
                                    let mut file = fs::File::create(&file_path)?;

                                    let mut response = reqwest::blocking::get(url.as_str())
                                        .map_err(|_| {
                                            ModLoaderWarning::download_failed(file_name.clone())
                                        })?;
                                    io::copy(&mut response, &mut file)?;

                                    Ok(())
                                })();
                                match download {
                                    Ok(_) => {}
                                    Err(err) => {
                                        warn!("Download error: {:?}", err);
                                        data.lock().warnings.push(err);
                                    }
                                }
                            }
                            // process newly downlaoded files
                            let warnings = process_modfiles(
                                &files_to_download
                                    .iter()
                                    .map(|f| mods_path.clone().join(f.0.clone()))
                                    .collect::<Vec<_>>(),
                                &data,
                                false,
                            );
                            debug!("warnings: {:?}", warnings);
                            data.lock().warnings.extend(warnings);
                        }

                        // move mods

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
                        }

                        let start = Instant::now();

                        // run integrator
                        debug!("Integrating mods");
                        integrate_mods(
                            config.get_integrator_config(),
                            &paks_path,
                            &install_path.join(D::GAME_NAME).join("Content").join("Paks"),
                            refuse_mismatched_connections,
                        )?;

                        debug!(
                            "Integration took {} milliseconds",
                            start.elapsed().as_millis()
                        );

                        *last_integration_time.lock() = Instant::now();

                        let mut data_guard = data.lock();

                        // update config file
                        write_config(&mut data_guard);

                        Ok(())
                    })();
                    match integration_work {
                        Ok(_) => {}
                        Err(err) => {
                            warn!("Integration work error: {:?}", err);
                            data.lock().warnings.push(err);
                        }
                    }

                    working.store(false, Ordering::Release);
                } else {
                    drop(data_guard);
                }

                thread::sleep(Duration::from_millis(50));
            }
        })
        .unwrap_or_else(|_| {
            error!("Failed to start background thread");
            panic!();
        });

    // run the GUI app
    eframe::run_native(
        app.window_title.clone().as_str(),
        eframe::NativeOptions {
            initial_window_size: Some(eframe::egui::vec2(623.0, 600.0)),
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
