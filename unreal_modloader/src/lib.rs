use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use log::{debug, error};
use unreal_modintegrator::{integrate_mods, IntegratorConfig};

mod app;
pub mod config;
mod determine_paths;
pub(crate) mod game_mod;
mod mod_config;
mod mod_processing;
pub mod version;

use game_mod::GameMod;
use mod_config::{load_config, write_config};
use mod_processing::process_modfiles;
use version::GameBuild;

#[derive(Debug)]
pub(crate) struct AppData {
    /// %LocalAppData%\[GameName]\Saved
    pub base_path: Option<PathBuf>,
    /// %LocalAppData%\[GameName]\Saved\Mods
    pub data_path: Option<PathBuf>,
    /// %LocalAppData%\[GameName]\Saved\Paks
    pub paks_path: Option<PathBuf>,
    /// install path
    pub install_path: Option<PathBuf>,

    pub game_build: Option<GameBuild>,
    pub refuse_mismatched_connections: bool,

    pub game_mods: BTreeMap<String, GameMod>,
}

pub fn run<'a, C, D, T: 'a, E: 'static + std::error::Error>(config: C)
where
    D: 'static + IntegratorConfig<'a, T, E>,
    C: 'static + config::GameConfig<'a, D, T, E>,
{
    let data = Arc::new(Mutex::new(AppData {
        base_path: None,
        data_path: None,
        paks_path: None,
        install_path: None,
        game_build: None,
        refuse_mismatched_connections: true,

        game_mods: BTreeMap::new(),
    }));

    let should_exit = Arc::new(AtomicBool::new(false));
    let ready_exit = Arc::new(AtomicBool::new(false));
    let should_integrate = Arc::new(AtomicBool::new(true));
    let working = Arc::new(AtomicBool::new(true));

    // instantiate the GUI app
    let app = app::App {
        data: Arc::clone(&data),
        window_title: config.get_window_title(),
        dropped_files: Vec::new(),

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
            data.lock().unwrap().base_path = determine_paths::dertermine_base_path(
                config.get_integrator_config().get_game_name().as_str(),
            );
            data.lock().unwrap().install_path =
                determine_paths::dertermine_install_path(config.get_app_id());

            if data.lock().unwrap().base_path.is_some() {
                let mut data_guard = data.lock().unwrap();

                // set sub dirs
                let base_path = data_guard.base_path.as_ref().unwrap().to_owned();
                data_guard.data_path = Some(PathBuf::from(base_path.clone()).join("Mods"));
                data_guard.paks_path = Some(PathBuf::from(base_path.clone()).join("Paks"));

                let data_path = data_guard.data_path.as_ref().unwrap().to_owned();
                drop(data_guard);

                // ensure the base_path/Mods directory exists
                fs::create_dir_all(&data_path).unwrap();

                // gather mods
                let mods_dir = fs::read_dir(&data_path).unwrap_or_else(|_| {
                    error!("Failed to read mods directory");
                    panic!();
                });

                let mod_files: Vec<PathBuf> = mods_dir
                    .filter_map(|e| e.ok())
                    .filter(|e| match e.file_name().into_string() {
                        Ok(s) => s.ends_with("_P.pak") && s != "999-Mods_P.pak",
                        Err(_) => false,
                    })
                    .map(|e| e.path())
                    .collect();

                process_modfiles(&mod_files, &data).unwrap();

                // load config
                let mut data_guard = data.lock().unwrap();
                load_config(&mut *data_guard);

                // debug!("{:#?}", data_guard.game_mods);
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

                let data_guard = data.lock().unwrap();
                if should_integrate.load(Ordering::Relaxed) && data_guard.base_path.is_some() {
                    working.store(true, Ordering::Relaxed);
                    should_integrate.store(false, Ordering::Relaxed);

                    // gather mods to be installed
                    let mods_to_install = data_guard
                        .game_mods
                        .iter()
                        .filter(|(_, m)| m.active)
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
                    let refuse_mismatched_connections = data_guard.refuse_mismatched_connections;
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

                    if files_to_downlaod.len() > 0 {
                        // ? Maybe parallelize this?
                        for (file_name, url) in &files_to_downlaod {
                            debug!("Downloading {:?}", file_name);

                            // this is safe because the filename has already been validated
                            let file_path =
                                PathBuf::from(mods_path.clone()).join(file_name.clone());
                            let mut file = fs::File::create(&file_path).unwrap();

                            let mut response = reqwest::blocking::get(url.as_str()).unwrap();
                            io::copy(&mut response, &mut file).unwrap();
                        }
                        // process newly downlaoded files
                        process_modfiles(
                            &files_to_downlaod
                                .iter()
                                .map(|f| PathBuf::from(mods_path.clone()).join(f.0.clone()))
                                .collect::<Vec<_>>(),
                            &data,
                        )
                        .unwrap();
                    }

                    // move mods
                    // remove all old files
                    fs::remove_dir_all(&paks_path).unwrap_or_else(|_| {
                        error!("Failed to remove paks directory");
                        panic!();
                    });
                    fs::create_dir(&paks_path).unwrap_or_else(|_| {
                        error!("Failed to create paks directory");
                        panic!();
                    });

                    // copy new files
                    for mod_version in mods_to_install {
                        fs::copy(
                            mods_path.join(mod_version.file_name.as_str()),
                            paks_path.join(mod_version.file_name.as_str()),
                        )
                        .unwrap_or_else(|_| {
                            error!("Failed to copy pak file {:?}", mod_version.file_name);
                            panic!();
                        });
                    }

                    // run integrator
                    debug!("Integrating mods");
                    integrate_mods(
                        config.get_integrator_config(),
                        &paks_path,
                        &install_path
                            .join(config.get_integrator_config().get_game_name())
                            .join("Content")
                            .join("Paks"),
                        refuse_mismatched_connections,
                    )
                    .unwrap();

                    let mut data_guard = data.lock().unwrap();

                    // update config file
                    write_config(&mut data_guard);

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
