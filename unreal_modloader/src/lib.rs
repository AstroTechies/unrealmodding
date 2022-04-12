use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::path::PathBuf;

mod app;
mod determine_paths;
pub mod config;

pub struct AppData {
    pub should_exit: bool,
    pub ready_exit: bool,

    pub base_path: Option<PathBuf>,
    pub install_path: Option<PathBuf>,
}

pub fn run<C, E>(config: C)
where
    E: config::DummyIntegratorConfig,
    C: 'static + config::GameConfig<E>,
{
    let config = Arc::new(Mutex::new(config));

    println!(
        "Got integrator config: {:?}",
        config.lock().unwrap().get_integrator_config().dummy()
    );

    let data = Arc::new(Mutex::new(AppData {
        should_exit: false,
        ready_exit: false,

        base_path: None,
        install_path: None,
    }));

    let data_clone = Arc::clone(&data);
    let config_clone = Arc::clone(&config);
    // spawn a background thread to handle long running tasks
    thread::spawn(move || {
        println!("Starting background thread");

        // shorthand alias
        let data = data_clone;
        let config = config_clone.lock().unwrap();


        let base_path = determine_paths::dertermine_base_path(config.get_game_name().as_str());
        let install_path = determine_paths::dertermine_install_path(config.get_app_id());
        data.lock().unwrap().base_path = base_path;
        data.lock().unwrap().install_path = install_path;

        loop {
            let mut data = data.lock().unwrap();
            if data.should_exit {
                println!("Background thread exiting...");
                data.ready_exit = true;
                break;
            }

            drop(data);
            thread::sleep(Duration::from_millis(500));
        }
    });

    let app = app::App {
        data,
        // TODO: this might be result in a deadlock if the background thread starts fast enough
        window_title: config.lock().unwrap().get_window_title(),
    };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
