use eframe::{egui, epi};
use std::sync::{Arc, Mutex};

pub struct AppData {
    pub should_exit: bool,
    pub ready_exit: bool,

    pub window_title: String,
}

pub struct App {
    pub data: Arc<Mutex<AppData>>,

    pub window_title: String,
}

impl epi::App for App {
    fn name(&self) -> &str {
        self.window_title.as_str()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let mut data = self.data.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.window_title.as_str());

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
        let mut data = self.data.lock().unwrap();
        data.should_exit = true;

        if data.ready_exit {
            println!("Exiting...");
        }

        data.ready_exit
    }
}
