use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod app;

pub fn run() {
    println!("Unreal Modloader");

    let data = Arc::new(Mutex::new(app::ModloaderData {
        should_exit: false,
        ready_exit: false,
    }));

    let data_processing = Arc::clone(&data);
    thread::spawn(move || {
        let data = data_processing;

        loop {
            let mut data = data.lock().unwrap();
            if data.should_exit {
                println!("Processing thread exiting...");
                data.ready_exit = true;
                break;
            }

            drop(data);
            thread::sleep(Duration::from_millis(10));
        }
    });

    let app = app::Modloader { data };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
