use eframe::{egui, epi};
use std::sync::{Arc, Mutex};

pub struct ModloaderData {
    pub should_exit: bool,
    pub ready_exit: bool,
}

pub struct Modloader {
    pub data: Arc<Mutex<ModloaderData>>,
}

impl epi::App for Modloader {
    fn name(&self) -> &str {
        "Modloader"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let Self { data } = self;
        let mut data = data.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Modloader");

            ui.horizontal(|ui| {
                if ui.button("Quit").clicked() {
                    frame.quit();
                }

                if data.should_exit {
                    ui.label("Exiting...");
                }
            });

            egui::warn_if_debug_build(ui);
        });

        // egui doesn't repaint when data is changed from a different thread,
        // so we migth have to constantly just repaint
        // frame.request_repaint();

        if data.should_exit && data.ready_exit {
            frame.quit();
        }
    }

    fn on_exit_event(&mut self) -> bool {
        let Self { data } = self;
        let mut data = data.lock().unwrap();
        data.should_exit = true;

        if data.ready_exit {
            println!("Exiting...");
        }

        data.ready_exit
    }
}
