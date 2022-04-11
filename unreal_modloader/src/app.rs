use eframe::{egui, epi};
use std::sync::{Arc, Mutex};

use crate::determine_paths;

pub struct App {
    pub data: Arc<Mutex<crate::AppData>>,

    pub window_title: String,
}

impl epi::App for App {
    fn name(&self) -> &str {
        self.window_title.as_str()
    }

    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        let mut data = self.data.lock().unwrap();
        determine_paths::dertermine_base_path(&mut data);
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

            ui.label(match data.base_path {
                Some(ref path) => path.to_str().unwrap(),
                None => "No base path",
            });

            egui::warn_if_debug_build(ui);
        });

        if data.base_path.is_none() {
            egui::Window::new("Can't find data directory").show(ctx, |ui| {
                ui.label("Failed to find local application data directory.");

                if ui.button("Quit").clicked() {
                    frame.quit();
                }
            });
        }

        // We need to keep the paint loop constantly running when shutting down,
        // otherwise the background thread might be done, but the paint loop is
        // in idle becasue there is no user input.
        if data.should_exit {
            frame.request_repaint();
        }

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
