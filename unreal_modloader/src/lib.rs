use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod app;
pub mod config;

pub fn run<C, E>(config: &C)
where
    E: config::DummyIntegratorConfig,
    C: config::GameConfig<E>,
{
    println!("Unreal Modloader");

    println!("Got integrator config: {:?}", config.get_integrator_config().dummy());

    let data = Arc::new(Mutex::new(app::AppData {
        should_exit: false,
        ready_exit: false,

        window_title: config.get_game_name(),
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
