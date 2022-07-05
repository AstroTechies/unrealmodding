use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use lazy_static::lazy_static;
use log::warn;
use log::{debug, error};
use regex::Regex;
use unreal_modintegrator::{integrate_mods, IntegratorConfig};

mod app;
pub mod config;
mod determine_paths;
pub mod error;
pub(crate) mod game_mod;
mod mod_config;
mod mod_processing;
pub mod version;

use error::{ModLoaderError, ModLoaderWarning};
use game_mod::GameMod;
use mod_config::{load_config, write_config};
use mod_processing::process_modfiles;
use version::GameBuild;

#[derive(Clone, Copy, Debug)]
pub(crate) enum GameFlavor {
    Steam,
    WinStore,
}

#[derive(Debug)]
pub(crate) struct ModLoaderAppData {
    /// %LocalAppData%\[GameName]\Saved
    pub base_path: Option<PathBuf>,
    /// %LocalAppData%\[GameName]\Saved\Mods
    pub data_path: Option<PathBuf>,
    /// %LocalAppData%\[GameName]\Saved\Paks
    pub paks_path: Option<PathBuf>,
    /// install path
    pub install_path: Option<PathBuf>,
    /// game flavor
    pub game_flavor: Option<GameFlavor>,

    pub game_build: Option<GameBuild>,
    pub refuse_mismatched_connections: bool,
    pub game_mods: BTreeMap<String, GameMod>,
    pub files_to_process: Vec<PathBuf>,

    pub error: Option<ModLoaderError>,
    pub warnings: Vec<ModLoaderWarning>,
}

lazy_static! {
    static ref APPX_MANIFEST_VERSION_REGEX: Regex =
        Regex::new("(?x)<Identity(.*?)Publisher(.*?)Version=\"([^\"]*)\"").unwrap();
}

pub fn run<'a, C, D, T: 'a, E: 'static + std::error::Error + Send>(config: C)
where
    D: 'static + IntegratorConfig<'a, T, E>,
    C: 'static + config::GameConfig<'a, D, T, E>,
{
    let data = Arc::new(Mutex::new(ModLoaderAppData {
        base_path: None,
        data_path: None,
        paks_path: None,
        install_path: None,
        game_flavor: None,

        game_build: None,
        refuse_mismatched_connections: true,
        game_mods: BTreeMap::new(),
        files_to_process: Vec::new(),

        error: None,
        warnings: Vec::new(),
    }));

    let should_exit = Arc::new(AtomicBool::new(false));
    let ready_exit = Arc::new(AtomicBool::new(false));
    let should_integrate = Arc::new(AtomicBool::new(true));
    let working = Arc::new(AtomicBool::new(true));

    // instantiate the GUI app
    let app = app::ModLoaderApp {
        data: Arc::clone(&data),
        window_title: C::WINDOW_TITLE.to_owned(),
        processed_files: HashSet::new(),

        should_exit: Arc::clone(&should_exit),
        ready_exit: Arc::clone(&ready_exit),
        should_integrate: Arc::clone(&should_integrate),
        working: Arc::clone(&working),
    };

    // spawn a background thread to handle long running tasks
    thread::Builder::new()
        .name("background".to_string())
        .spawn(move || {
            debug!("Starting background thread");

            // startup work
            let start = Instant::now();

            // get paths
            let mut base_path: Option<PathBuf> = None;

            let install_path = determine_paths::determine_install_path_steam(C::APP_ID);
            if install_path.is_ok()
                && determine_paths::verify_install_path(
                    install_path.as_ref().unwrap(),
                    D::GAME_NAME,
                )
            {
                data.lock().unwrap().install_path = install_path.ok();
                data.lock().unwrap().game_flavor = Some(GameFlavor::Steam);
                base_path = determine_paths::determine_base_path_steam(D::GAME_NAME);
            } else {
                // checking winstore
                if let Some(winstore_vendor_id) = C::WINSTORE_VENDOR_ID {
                    let install_path =
                        determine_paths::determine_install_path_winstore(winstore_vendor_id);
                    if let Ok(install_path) = install_path {
                        //todo: take ownership
                        data.lock().unwrap().install_path = Some(install_path.path.clone());
                        data.lock().unwrap().game_flavor = Some(GameFlavor::WinStore);
                        base_path = determine_paths::determine_base_path_winstore(
                            &install_path,
                            D::GAME_NAME,
                        );
                    } else {
                        data.lock()
                            .unwrap()
                            .warnings
                            .push(install_path.unwrap_err());
                    }
                } else {
                    data.lock()
                        .unwrap()
                        .warnings
                        .push(install_path.unwrap_err());
                }
            }

            if base_path.is_none() {
                error!("Could not determine base path");
                data.lock().unwrap().error = Some(ModLoaderError::no_base_path());
            } else {
                data.lock().unwrap().base_path = base_path;
            }

            if data.lock().unwrap().base_path.is_some() {
                let mut data_guard = data.lock().unwrap();

                // set sub dirs
                let base_path = data_guard.base_path.as_ref().unwrap().to_owned();
                data_guard.data_path = Some(base_path.join("Mods"));
                data_guard.paks_path = Some(base_path.join("Paks"));

                let data_path = data_guard.data_path.as_ref().unwrap().to_owned();
                let paks_path = data_guard.paks_path.as_ref().unwrap().to_owned();
                drop(data_guard);

                let startup_work = || -> Result<(), ModLoaderError> {
                    // ensure the base_path/Mods directory exists
                    fs::create_dir_all(&data_path).map_err(|err| {
                        ModLoaderError::io_error_with_message("Mods directory".to_owned(), err)
                    })?;
                    // ensure /Paks exists
                    fs::create_dir_all(&paks_path).map_err(|err| {
                        ModLoaderError::io_error_with_message("Paks directory".to_owned(), err)
                    })?;

                    // gather mods
                    let mods_dir = fs::read_dir(&data_path).map_err(|err| {
                        ModLoaderError::io_error_with_message("Mods directory".to_owned(), err)
                    })?;

                    let mod_files: Vec<PathBuf> = mods_dir
                        .filter_map(|e| e.ok())
                        .filter(|e| match e.file_name().into_string() {
                            Ok(s) => s.ends_with("_P.pak") && s != "999-Mods_P.pak",
                            Err(_) => false,
                        })
                        .map(|e| e.path())
                        .collect();

                    let warnings = process_modfiles(&mod_files, &data, false);
                    debug!("warnings: {:?}", warnings);

                    let mut data_guard = data.lock().unwrap();
                    data_guard.warnings.extend(warnings);

                    // load config
                    load_config(&mut *data_guard, D::GAME_NAME);

                    // debug!("{:#?}", data_guard.game_mods);
                    Ok(())
                };
                match startup_work() {
                    Ok(_) => {}
                    Err(err) => {
                        data.lock().unwrap().error = Some(err);
                    }
                }
            }

            debug!(
                "Background thread startup took {} milliseconds",
                start.elapsed().as_millis()
            );

            working.store(false, Ordering::Relaxed);

            // background loop
            loop {
                if should_exit.load(Ordering::Relaxed) {
                    debug!("Background thread exiting...");
                    ready_exit.store(true, Ordering::Relaxed);
                    break;
                }

                // process dropped files
                let mut data_guard = data.lock().unwrap();
                if data_guard.base_path.is_some() && !data_guard.files_to_process.is_empty() {
                    let files_to_process = data_guard
                        .files_to_process
                        .clone()
                        .iter()
                        .filter_map(|file_path| {
                            let file_name = file_path.file_name().unwrap();

                            // copy the file to the mods directory
                            let new_file_path =
                                data_guard.data_path.as_ref().unwrap().join(file_name);
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
                    data.lock().unwrap().warnings.extend(warnings);

                    should_integrate.store(true, Ordering::Relaxed);
                } else {
                    drop(data_guard);
                }

                let mut data_guard = data.lock().unwrap();
                if should_integrate.load(Ordering::Relaxed)
                    && data_guard.base_path.is_some()
                    && data_guard.install_path.is_some()
                    && data_guard.warnings.is_empty()
                {
                    let integration_work = (|| -> Result<(), ModLoaderWarning> {
                        working.store(true, Ordering::Relaxed);
                        should_integrate.store(false, Ordering::Relaxed);

                        // set game build
                        if data_guard.install_path.is_some() {
                            if data_guard.game_flavor.is_some() {
                                let flavor = data_guard.game_flavor.as_ref().unwrap();
                                match flavor {
                                    GameFlavor::Steam => {
                                        data_guard.game_build = config.get_game_build(
                                            data_guard.install_path.as_ref().unwrap(),
                                        );
                                    }
                                    GameFlavor::WinStore => {
                                        // can parse it for any winstore app
                                        if let Ok(mut appx_file) = File::open(
                                            data_guard
                                                .install_path
                                                .as_ref()
                                                .unwrap()
                                                .join("AppxManifest.xml"),
                                        ) {
                                            let mut appx_contents = String::new();
                                            appx_file.read_to_string(&mut appx_contents)?;

                                            let app_version = APPX_MANIFEST_VERSION_REGEX
                                                .captures(&appx_contents);

                                            let version_capture =
                                                app_version.map(|e| e.get(e.len() - 1)).flatten();
                                            if version_capture.is_some() {
                                                let version = version_capture.unwrap().as_str();
                                                let version_numbers: Vec<usize> = version
                                                    .split(".")
                                                    .filter_map(|e| e.parse::<usize>().ok())
                                                    .collect();
                                                if version_numbers.len() == 4 {
                                                    data_guard.game_build = Some(GameBuild::new(
                                                        version_numbers[0],
                                                        version_numbers[1],
                                                        version_numbers[2],
                                                        version_numbers[3],
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                // maybe error out? this should never be reached
                                data_guard.game_build = config
                                    .get_game_build(data_guard.install_path.as_ref().unwrap());
                            }
                        }

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

                        let mods_path = data_guard.data_path.as_ref().unwrap().to_owned();
                        let paks_path = data_guard.paks_path.as_ref().unwrap().to_owned();
                        let install_path = data_guard.install_path.as_ref().unwrap().to_owned();
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
                        let files_to_downlaod: Vec<(String, String)> = mods_to_install
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

                        if !files_to_downlaod.is_empty() {
                            // ? Maybe parallelize this?
                            for (file_name, url) in &files_to_downlaod {
                                let downlaod = (|| -> Result<(), ModLoaderWarning> {
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
                                match downlaod {
                                    Ok(_) => {}
                                    Err(err) => {
                                        warn!("Download error: {:?}", err);
                                        data.lock().unwrap().warnings.push(err);
                                    }
                                }
                            }
                            // process newly downlaoded files
                            let warnings = process_modfiles(
                                &files_to_downlaod
                                    .iter()
                                    .map(|f| mods_path.clone().join(f.0.clone()))
                                    .collect::<Vec<_>>(),
                                &data,
                                false,
                            );
                            debug!("warnings: {:?}", warnings);
                            data.lock().unwrap().warnings.extend(warnings);
                        }

                        // move mods
                        // remove all old files
                        fs::remove_dir_all(&paks_path)?;
                        fs::create_dir(&paks_path)?;

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

                        let mut data_guard = data.lock().unwrap();

                        // update config file
                        write_config(&mut data_guard);

                        Ok(())
                    })();
                    match integration_work {
                        Ok(_) => {}
                        Err(err) => {
                            warn!("Integration work error: {:?}", err);
                            data.lock().unwrap().warnings.push(err);
                        }
                    }

                    working.store(false, Ordering::Relaxed);
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
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.iter_mut().for_each(|font| {
                font.1.tweak.scale = 1.2;
            });
            cc.egui_ctx.set_fonts(fonts);

            cc.egui_ctx.set_style(egui::Style::default());

            Box::new(app)
        }),
    );
}
