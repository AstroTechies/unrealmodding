use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use eframe::egui;
use unreal_modintegrator::IntegratorConfig;

mod app;
pub mod config;
mod determine_paths;
pub(crate) mod game_mod;
mod mod_config;
mod mod_processing;
pub mod version;

use game_mod::GameMod;
use mod_config::load_config;
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

    pub game_mods: BTreeMap<String, GameMod>,
}

pub fn run<'a, C, D, T, E: std::error::Error>(config: C)
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
            println!("Starting background thread");

            // startup work
            let start = Instant::now();

            // get paths
            data.lock().unwrap().base_path =
                determine_paths::dertermine_base_path(config.get_game_name().as_str());
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

                // TODO: better error handling for all of this
                // gather mods
                let mods_dir = fs::read_dir(&data_path).unwrap();

                let mod_files: Vec<fs::DirEntry> = mods_dir
                    .filter_map(|e| e.ok())
                    .filter(|e| match e.file_name().into_string() {
                        Ok(s) => s.ends_with("_P.pak") && s != "999-Mods_P.pak",
                        Err(_) => false,
                    })
                    .collect();

                process_modfiles(&mod_files, &data).unwrap();

                // load config
                let mut data_guard = data.lock().unwrap();
                load_config(&mut *data_guard);

                //println!("{:#?}", data_guard.game_mods);
            }

            println!(
                "Background thread startup took {} milliseconds",
                start.elapsed().as_millis()
            );

            working.store(false, Ordering::Relaxed);

            // background loop
            loop {
                if should_exit.load(Ordering::Relaxed) {
                    println!("Background thread exiting...");
                    ready_exit.store(true, Ordering::Relaxed);
                    break;
                }

                let data = data.lock().unwrap();
                if should_integrate.load(Ordering::Relaxed) && data.base_path.is_some() {
                    drop(data);
                    println!(
                        "Integrating mods with config engine_version: {:?}",
                        config.get_integrator_config().get_engine_version()
                    );

                    working.store(true, Ordering::Relaxed);
                    should_integrate.store(false, Ordering::Relaxed);

                    // TODO: move mods
                    // TODO: run integrator

                    working.store(false, Ordering::Relaxed);
                } else {
                    drop(data);
                }

                thread::sleep(Duration::from_millis(50));
            }
        })
        .expect("Failure to spawn background thread");

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
