use eframe::{egui, epi};
use egui_extras::{Size, StripBuilder, TableBuilder};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub(crate) struct App {
    pub data: Arc<Mutex<crate::AppData>>,

    pub window_title: String,
    pub dropped_files: Vec<egui::DroppedFile>,

    pub should_exit: Arc<AtomicBool>,
    pub ready_exit: Arc<AtomicBool>,

    pub should_integrate: Arc<AtomicBool>,
    pub working: Arc<AtomicBool>,
}

impl epi::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut epi::Frame) {
        let mut data = self.data.lock().unwrap();
        let should_integrate = &self.should_integrate;

        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(30.0))
                .size(Size::exact(200.0))
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        if !self.working.load(Ordering::Relaxed) {
                            ui.heading(self.window_title.as_str());
                        } else {
                            ui.heading(format!("{} - Working...", self.window_title.as_str()));
                        }
                    });
                    strip.cell(|ui| {
                        TableBuilder::new(ui)
                            .striped(true)
                            .cell_layout(
                                egui::Layout::left_to_right().with_cross_align(egui::Align::Center),
                            )
                            .column(Size::initial(42.0).at_least(40.0))
                            .column(Size::initial(200.0).at_least(20.0))
                            .column(Size::initial(60.0).at_least(20.0))
                            .column(Size::initial(70.0).at_least(20.0))
                            .column(Size::remainder().at_least(20.0))
                            .resizable(true)
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Active");
                                });
                                header.col(|ui| {
                                    ui.strong("Name");
                                });
                                header.col(|ui| {
                                    ui.strong("Version");
                                });
                                header.col(|ui| {
                                    ui.strong("Author");
                                });
                                header.col(|ui| {
                                    ui.strong("Game build");
                                });
                            })
                            .body(|mut body| {
                                for (_, game_mod) in data.game_mods.iter_mut() {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| {
                                            if ui.checkbox(&mut game_mod.active, "").changed() {
                                                should_integrate.store(true, Ordering::Relaxed);
                                            };
                                        });
                                        row.col(|ui| {
                                            ui.label(game_mod.name.as_str());
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{}", game_mod.selected_version));
                                        });
                                        row.col(|ui| {
                                            ui.label(
                                                game_mod
                                                    .author
                                                    .as_ref()
                                                    .unwrap_or(&"No author".to_owned())
                                                    .as_str(),
                                            );
                                        });
                                        row.col(|ui| {
                                            let temp: String;
                                            ui.label(match game_mod.game_build {
                                                Some(ref b) => {
                                                    temp = b.to_string();
                                                    temp.as_str()
                                                }
                                                None => "Any",
                                            });
                                        });
                                    });
                                }
                            });
                    });
                    strip.cell(|ui| {
                        ui.label("Mod config");
                        ui.label("TODO");

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

                        egui::warn_if_debug_build(ui);
                    });
                });
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
            ctx.request_repaint();
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
