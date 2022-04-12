use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

mod app;
pub mod config;
mod determine_paths;
mod game_mod;
pub mod version;

use game_mod::{GameMod, GameModVersion, SelectedVersion};
use version::{GameBuild, Version};

pub struct AppData {
    pub base_path: Option<PathBuf>,
    pub install_path: Option<PathBuf>,

    pub game_mods: Vec<game_mod::GameMod>,
}

pub fn run<C, E>(config: C)
where
    E: config::DummyIntegratorConfig,
    C: 'static + config::GameConfig<E>,
{
    println!(
        "Got integrator config: {:?}",
        config.get_integrator_config().dummy()
    );

    // TODO: remove temp test
    let test_mod = GameMod {
        mod_id: "TestMod".to_string(),

        versions: vec![GameModVersion {
            version: Version {
                major: 1,
                minor: 0,
                patch: 0,
            },
            file_name: "000-TestMod-1.0.0_P.pak".to_string(),
            downloaded: true,
        }],
        latest_version: None,
        selected_version: SelectedVersion::Specific(Version::new(1, 0, 0)),

        active: true,

        name: "Test Mod".to_string(),
        author: "Konsti".to_string(),
        description: "test mod description".to_string(),
        game_build: GameBuild::new(1, 24, 29, 0),
        sync: None,
        homepage: "https://astroneermods.space/m/TestMod".to_string(),
        download: None,
        size: 1000,
    };

    let data = Arc::new(Mutex::new(AppData {
        base_path: None,
        install_path: None,

        game_mods: vec![test_mod],
    }));

    let should_exit = Arc::new(AtomicBool::new(false));
    let ready_exit = Arc::new(AtomicBool::new(false));
    let should_integrate = Arc::new(AtomicBool::new(false));
    let working = Arc::new(AtomicBool::new(false));

    // instantiate the GUI app
    let app = app::App {
        data: Arc::clone(&data),
        window_title: config.get_window_title(),

        should_exit: Arc::clone(&should_exit),
        ready_exit: Arc::clone(&ready_exit),
        should_integrate: Arc::clone(&should_integrate),
        working: Arc::clone(&working),
    };

    // spawn a background thread to handle long running tasks
    thread::spawn(move || {
        println!("Starting background thread");

        // startup work
        working.store(true, Ordering::Relaxed);

        // get paths
        let base_path = determine_paths::dertermine_base_path(config.get_game_name().as_str());
        let install_path = determine_paths::dertermine_install_path(config.get_app_id());
        data.lock().unwrap().base_path = base_path;
        data.lock().unwrap().install_path = install_path;

        working.store(true, Ordering::Relaxed);

        // background loop
        loop {
            let mut data = data.lock().unwrap();
            if should_exit.load(Ordering::Relaxed) {
                println!("Background thread exiting...");
                ready_exit.store(true, Ordering::Relaxed);
                break;
            }

            if should_integrate.load(Ordering::Relaxed) {
                println!("Integrating mods...");
                working.store(true, Ordering::Relaxed);
                should_integrate.store(false, Ordering::Relaxed);

                // TODO: move mods
                // TODO: run integrator

                working.store(false, Ordering::Relaxed);
            }

            drop(data);
            thread::sleep(Duration::from_millis(50));
        }
    });

    // run the GUI app
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
