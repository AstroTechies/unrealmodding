use eframe::{egui, epi};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

pub struct App {
    pub data: Arc<Mutex<crate::AppData>>,

    pub window_title: String,

    pub should_exit: Arc<AtomicBool>,
    pub ready_exit: Arc<AtomicBool>,

    pub should_integrate: Arc<AtomicBool>,
    pub working: Arc<AtomicBool>,
}

impl epi::App for App {
    fn name(&self) -> &str {
        self.window_title.as_str()
    }

    // fn setup(
    //     &mut self,
    //     _ctx: &egui::Context,
    //     _frame: &epi::Frame,
    //     _storage: Option<&dyn epi::Storage>,
    // ) {

    // }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let mut data = self.data.lock().unwrap();
        let should_integrate = &self.should_integrate;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.window_title.as_str());

            ui.horizontal(|ui| {
                if ui.button("Quit").clicked() {
                    frame.quit();
                }

                if self.should_exit.load(Ordering::Relaxed) {
                    ui.label("Exiting...");
                }
            });

            ui.label(match data.base_path {
                Some(ref path) => path.to_str().unwrap(),
                None => "No base path",
            });

            ui.label(match data.install_path {
                Some(ref path) => path.to_str().unwrap(),
                None => "No install path",
            });

            ui.heading("Mods");
            for game_mod in data.game_mods.iter_mut() {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut game_mod.active, "Active").changed() {
                        should_integrate.store(true, Ordering::Relaxed);
                    };
                    ui.label(game_mod.name.as_str());
                    ui.label(game_mod.author.as_str());
                    ui.label(game_mod.game_build.to_string().as_str());
                });
            }


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
        if self.should_exit.load(Ordering::Relaxed) {
            frame.request_repaint();
        }

        if self.should_exit.load(Ordering::Relaxed) && self.ready_exit.load(Ordering::Relaxed) {
            frame.quit();
        }
    }

    fn on_exit_event(&mut self) -> bool {
        self.should_exit.store(true, Ordering::Relaxed);

        if self.ready_exit.load(Ordering::Relaxed) {
            println!("Exiting...");
        }

        self.ready_exit.load(Ordering::Relaxed)
    }
}
