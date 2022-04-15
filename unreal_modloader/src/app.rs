use eframe::{egui, epi};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub struct App {
    pub data: Arc<Mutex<crate::AppData>>,

    pub window_title: String,
    pub dropped_files: Vec<egui::DroppedFile>,

    pub should_exit: Arc<AtomicBool>,
    pub ready_exit: Arc<AtomicBool>,

    pub should_integrate: Arc<AtomicBool>,
    pub working: Arc<AtomicBool>,
}

impl epi::App for App {
    fn name(&self) -> &str {
        self.window_title.as_str()
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.iter_mut().for_each(|font| {
            font.1.tweak.scale = 1.2;
        });

        ctx.set_fonts(fonts);
    }

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
            egui::Grid::new("mods_table").show(ui, |ui| {
                // header
                ui.label("Active");
                ui.label("Name");
                ui.label("Version");
                ui.label("Author");
                ui.label("Game build");
                ui.end_row();

                for game_mod in data.game_mods.iter_mut() {
                    if ui.checkbox(&mut game_mod.active, "").changed() {
                        should_integrate.store(true, Ordering::Relaxed);
                    };
                    ui.label(game_mod.name.as_str());
                    ui.label("Version here");
                    ui.label(game_mod.author.as_str());
                    ui.label(game_mod.game_build.to_string().as_str());
                    ui.end_row();
                }
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
        drop(data);

        self.detect_files_being_dropped(ctx);

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

impl App {
    fn detect_files_being_dropped(&mut self, ctx: &egui::Context) {
        use egui::*;

        // Preview hovering files:
        if !ctx.input().raw.hovered_files.is_empty() {
            let mut text = "Dropping files:\n".to_owned();
            for file in &ctx.input().raw.hovered_files {
                if let Some(path) = &file.path {
                    text += &format!("\n{}", path.display());
                } else if !file.mime.is_empty() {
                    text += &format!("\n{}", file.mime);
                } else {
                    text += "\n???";
                }
            }

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
        }
    }
}
