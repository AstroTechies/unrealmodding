use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use eframe::{egui, App};
use egui_extras::{Size, StripBuilder, TableBuilder};
use log::info;

use crate::game_mod::{GameMod, SelectedVersion};
use crate::version::Version;
use crate::ModLoaderAppData;

pub(crate) struct ModLoaderApp {
    pub data: Arc<Mutex<crate::ModLoaderAppData>>,

    pub window_title: String,
    pub dropped_files: Vec<egui::DroppedFile>,

    pub should_exit: Arc<AtomicBool>,
    pub ready_exit: Arc<AtomicBool>,

    pub should_integrate: Arc<AtomicBool>,
    pub working: Arc<AtomicBool>,
}

impl App for ModLoaderApp {
    fn update(&mut self, ctx: &egui::Context, mut frame: &mut eframe::Frame) {
        let mut data = self.data.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(30.0))
                .size(Size::relative(0.45))
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        self.show_title(ui, &mut data);
                    });
                    strip.cell(|ui| {
                        self.show_table(ui, &mut data);
                    });
                    strip.cell(|ui| {
                        self.show_bottom(ui, &mut data, &mut frame);
                    });
                });
        });

        let mut should_darken = false;
        if data.error.is_some() {
            egui::Window::new("Critical Error")
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
                .fixed_size((600.0, 400.0))
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(10.0, 25.0);

                    ui.label(format!("{}", data.error.as_ref().unwrap()));

                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

            should_darken = true;
        } else if data.warnings.len() > 0 {
            egui::Window::new("Warning")
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
                .fixed_size((600.0, 400.0))
                .show(ctx, |ui| {
                    //ui.spacing_mut().item_spacing = egui::vec2(10.0, 25.0);

                    //ui.label(format!("{}", data.error.as_ref().unwrap()));
                    for warning in &data.warnings {
                        ui.label(format!("{}", warning));
                    }

                    ui.label("");
                    ui.label("See modloader_log.txt for more details.");
                    ui.label("");

                    if ui.button("Ok").clicked() {
                        data.warnings.clear();
                    }
                });

            should_darken = true;
        }

        drop(data);

        if should_darken {
            self.darken_background(ctx);
        }

        self.detect_files_being_dropped(ctx);

        // We need to keep the paint loop constantly running when shutting down,
        // otherwise the background thread might be done, but the paint loop is
        // in idle becasue there is no user input.
        // Or keep it running while the background is actively working.
        if self.should_exit.load(Ordering::Relaxed) || self.working.load(Ordering::Relaxed) {
            ctx.request_repaint();
        }

        if self.should_exit.load(Ordering::Relaxed) && self.ready_exit.load(Ordering::Relaxed) {
            frame.quit();
        }
    }

    fn on_exit_event(&mut self) -> bool {
        self.should_exit.store(true, Ordering::Relaxed);

        if self.ready_exit.load(Ordering::Relaxed) {
            info!("Exiting...");
        }

        self.ready_exit.load(Ordering::Relaxed)
    }
}

impl ModLoaderApp {
    fn show_title(&self, ui: &mut egui::Ui, data: &mut ModLoaderAppData) {
        let title = format!(
            "Mods ({})",
            match data.game_build {
                Some(ref build) => build.to_string(),
                None => "<unknown>".to_owned(),
            }
        );
        if !self.working.load(Ordering::Relaxed) {
            ui.heading(title);
        } else {
            ui.heading(format!("{} - Working...", title));
        }
    }

    fn show_table(&self, ui: &mut egui::Ui, data: &mut ModLoaderAppData) {
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
            .column(Size::initial(42.0).at_least(42.0))
            .column(Size::initial(170.0).at_least(20.0))
            .column(Size::initial(120.0).at_least(120.0))
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
                for (_, mut game_mod) in data.game_mods.iter_mut() {
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            if ui.checkbox(&mut game_mod.active, "").changed() {
                                self.should_integrate.store(true, Ordering::Relaxed);
                            };
                        });
                        row.col(|ui| {
                            ui.label(game_mod.name.as_str());
                        });
                        row.col(|ui| {
                            // becasue ComboBox .chnaged doesn't seem to work
                            let prev_selceted = game_mod.selected_version.clone();

                            self.show_version_select(ui, &mut game_mod);

                            // this may look dumb but is what is needed
                            if prev_selceted != game_mod.selected_version {
                                self.should_integrate.store(true, Ordering::Relaxed);
                            }
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
    }

    fn show_version_select(&self, ui: &mut egui::Ui, game_mod: &mut GameMod) {
        egui::ComboBox::from_id_source(&game_mod.name)
            .selected_text(format!("{}", game_mod.selected_version))
            .show_ui(ui, |ui| {
                // for when there is an Index file show force latest version, this to diecrtly indicate that there
                // is the possibility of an auto update vie an index file.
                if game_mod.download.is_some() {
                    let latest_version = game_mod.latest_version.unwrap();
                    ui.selectable_value(
                        &mut game_mod.selected_version,
                        SelectedVersion::Latest(latest_version.clone()),
                        format!("{}", SelectedVersion::Latest(latest_version)),
                    );
                }

                // add all other versions to the drop down
                for version in game_mod.versions.iter() {
                    // if the version is the latest version, set it as LatestIndirect so that if there is an upgrade it will
                    // automatically be upgraded. This is under the assumption that if the user now has the latest version,
                    // that they probably also want to have the latest in the future.
                    let is_latest =
                        *version.0 == game_mod.latest_version.unwrap_or(Version::new(0, 0, 0));

                    let show_version = if is_latest {
                        SelectedVersion::LatestIndirect(Some(version.0.clone()))
                    } else {
                        SelectedVersion::Specific(version.0.clone())
                    };

                    ui.selectable_value(
                        &mut game_mod.selected_version,
                        show_version,
                        format!("{}", show_version),
                    );
                }
            });
    }

    fn show_bottom(
        &self,
        ui: &mut egui::Ui,
        data: &mut ModLoaderAppData,
        frame: &mut eframe::Frame,
    ) {
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

        if ui
            .checkbox(
                &mut data.refuse_mismatched_connections,
                "Refuse mismatched connections",
            )
            .changed()
        {
            self.should_integrate.store(true, Ordering::Relaxed);
        };

        egui::warn_if_debug_build(ui);
    }

    fn darken_background(&mut self, ctx: &egui::Context) {
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::PanelResizeLine,
            egui::Id::new("panel_darken"),
        ));

        let screen_rect = ctx.input().screen_rect();
        painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(192));
    }

    // from https://github.com/emilk/egui/blob/master/examples/file_dialog/src/main.rs
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
