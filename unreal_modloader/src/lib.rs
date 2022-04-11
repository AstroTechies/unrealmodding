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

pub fn run<C, E>(config: &C)
where
    E: config::DummyIntegratorConfig,
    C: config::GameConfig<E>,
{
    println!(
        "Got integrator config: {:?}",
        config.get_integrator_config().dummy()
    );

    let data = Arc::new(Mutex::new(AppData {
        should_exit: false,
        ready_exit: false,

        base_path: None,
        install_path: None,
    }));

    let data_processing = Arc::clone(&data);
    // spawn a background thread to handle long running tasks
    thread::spawn(move || {
        let data = data_processing;

        loop {
            let mut data = data.lock().unwrap();
            if data.should_exit {
                println!("Background thread exiting...");
                data.ready_exit = true;
                break;
            }

            drop(data);
            thread::sleep(Duration::from_millis(50));
        }
    });

    let app = app::App {
        data,
        window_title: config.get_window_title(),
    };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
