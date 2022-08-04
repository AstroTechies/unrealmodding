use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    Arc,
};
use std::time::Instant;

use eframe::egui::{Button, ProgressBar, Sense, Widget};
use eframe::emath::Align;
use eframe::{egui, App};
use egui_extras::{Size, StripBuilder, TableBuilder};
use log::{debug, info};
use parking_lot::Mutex;

use crate::version::Version;
use crate::{
    game_mod::{GameMod, SelectedVersion},
    update_info::UpdateInfo,
};

pub(crate) struct ModLoaderApp {
    pub data: Arc<Mutex<crate::ModLoaderAppData>>,

    pub window_title: String,

    pub should_exit: Arc<AtomicBool>,
    pub ready_exit: Arc<AtomicBool>,

    pub should_integrate: Arc<AtomicBool>,
    pub last_integration_time: Arc<Mutex<Instant>>,

    pub working: Arc<AtomicBool>,

    pub platform_selector_open: bool,
    pub selected_mod_id: Option<String>,

    pub newer_update: Arc<Mutex<Option<UpdateInfo>>>,
    pub should_update: Arc<AtomicBool>,
    pub update_progress: Arc<AtomicI32>,
}

impl App for ModLoaderApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(22.0))
                .size(Size::relative(0.45))
                .size(Size::remainder())
                .size(Size::exact(45.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        StripBuilder::new(ui)
                            .size(Size::relative(0.5))
                            .size(Size::remainder())
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    self.show_title(ui);
                                });
                                strip.cell(|ui| {
                                    self.show_change_platform(ui);
                                });
                            });

                        ui.separator();
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        self.show_table(ui);
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        self.show_bottom(ui);
                    });

                    strip.cell(|ui| {
                        ui.separator();
                        self.show_footer(ui);
                    });
                });
        });

        let mut data = self.data.lock();

        let mut should_darken = false;

        let mut update_cancelled = false;
        let newer_update = self.newer_update.lock();
        if let Some(newer_update) = newer_update.as_ref() {
            egui::Window::new("A new update is available")
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
                .default_size((600.0, 400.0))
                .show(ctx, |ui| {
                    StripBuilder::new(ui)
                        .size(Size::exact(22.0))
                        .size(Size::remainder())
                        .size(Size::exact(22.0))
                        .size(Size::exact(45.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                ui.heading(format!(
                                    "Update version {} is available!",
                                    newer_update.version
                                ));
                            });

                            strip.cell(|ui| {
                                ui.label(format!("Changelog:\n {}", newer_update.changelog));
                            });

                            strip.cell(|ui| {
                                if self.should_update.load(Ordering::Acquire) {
                                    let bar = ProgressBar::new(
                                        self.update_progress.load(Ordering::Acquire) as f32 / 100.0,
                                    );
                                    bar.ui(ui);
                                }
                            });

                            strip.cell(|ui| {
                                ui.separator();
                                ui.style_mut().spacing.button_padding = egui::vec2(9.0, 6.0);
                                ui.style_mut()
                                    .text_styles
                                    .get_mut(&egui::TextStyle::Button)
                                    .unwrap()
                                    .size = 16.0;

                                ui.with_layout(ui.layout().with_cross_align(Align::Center), |ui| {
                                    StripBuilder::new(ui)
                                        .size(Size::relative(0.5))
                                        .size(Size::remainder())
                                        .horizontal(|mut strip| {
                                            strip.cell(|ui| {
                                                if ui.button("Download").clicked() {
                                                    self.should_update
                                                        .store(true, Ordering::Release);
                                                }
                                            });

                                            strip.cell(|ui| {
                                                if ui.button("Cancel").clicked() {
                                                    update_cancelled = true;
                                                }
                                            });
                                        });
                                });
                            });
                        });
                });

            should_darken = true;
        }
        drop(newer_update);

        if update_cancelled {
            *self.newer_update.lock() = None;
        }

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
        } else if !data.warnings.is_empty() {
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

        if self.platform_selector_open {
            egui::Window::new("Platform Selector")
                .resizable(true)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
                .fixed_size((600.0, 400.0))
                .show(ctx, |ui| {
                    let key_count = data.install_managers.len();
                    for i in 0..key_count {
                        let platform = (*data.install_managers.keys().nth(i).unwrap()).to_string();
                        let manager = data.install_managers.get(platform.as_str()).unwrap();
                        let exists = manager.get_game_install_path().is_some();

                        let button = match exists {
                            true => Button::new(platform.to_string()),
                            false => Button::new(format!("{} (not found)", platform))
                                .sense(Sense::hover()),
                        };

                        if ui.add(button).clicked() {
                            data.set_game_platform(&platform);
                            self.platform_selector_open = false;
                            self.should_integrate.store(true, Ordering::Release);
                            ctx.request_repaint();
                        }
                    }
                });
            should_darken = true;
        }

        // Keyboard shortcuts

        // esc show default bottom text
        if ctx.input().key_pressed(egui::Key::Escape) {
            self.selected_mod_id = None;
        }

        // delete to remove a mod
        if ctx.input().key_pressed(egui::Key::Delete) {
            if let Some(ref id) = self.selected_mod_id {
                data.game_mods.get_mut(id).unwrap().remove = true;
                self.selected_mod_id = None;
                self.should_integrate.store(true, Ordering::Release);
            }
        }

        drop(data);

        if should_darken {
            self.darken_background(ctx);
        }

        self.detect_files_being_dropped(ctx);

        // We need to keep the paint loop constantly running when shutting down,
        // otherwise the background thread might be done, but the paint loop is
        // in idle becasue there is no user input.
        // Or keep it running while the background thread is actively working.
        // Or while integration is pending.
        // Or while the last integration was not long ago.
        if self.should_exit.load(Ordering::Acquire)
            || self.working.load(Ordering::Acquire)
            || self.should_integrate.load(Ordering::Acquire)
            || self.last_integration_time.lock().elapsed().as_secs() < 5
        {
            ctx.request_repaint();
        }

        if self.should_exit.load(Ordering::Acquire) && self.ready_exit.load(Ordering::Acquire) {
            frame.quit();
        }
    }

    fn on_exit_event(&mut self) -> bool {
        self.should_exit.store(true, Ordering::Release);

        if self.ready_exit.load(Ordering::Acquire) {
            info!("Exiting...");
        }

        self.ready_exit.load(Ordering::Acquire)
    }
}

impl ModLoaderApp {
    fn show_title(&self, ui: &mut egui::Ui) {
        let data = self.data.lock();

        let title = format!(
            "Mods ({})",
            match data.game_build {
                Some(ref build) => build.to_string(),
                None => "<unknown>".to_owned(),
            }
        );
        if !self.working.load(Ordering::Acquire) {
            ui.heading(title);
        } else {
            ui.heading(format!("{} - Working...", title));
        }
    }

    fn show_change_platform(&mut self, ui: &mut egui::Ui) {
        let data = self.data.lock();

        let title = format!(
            "Platform: {}",
            match data.selected_game_platform {
                Some(ref platform) => platform.to_string(),
                None => "None".to_owned(),
            },
        );

        ui.with_layout(ui.layout().with_cross_align(Align::Max), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Change platform").clicked() {
                    self.platform_selector_open = true;
                }
                ui.label(title);
            });
        });
    }

    fn show_table(&mut self, ui: &mut egui::Ui) {
        let mut data = self.data.lock();

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
            .column(Size::initial(42.0).at_least(42.0))
            .column(Size::initial(170.0).at_least(20.0))
            .column(Size::initial(120.0).at_least(120.0))
            .column(Size::initial(70.0).at_least(20.0))
            .column(Size::initial(80.0).at_least(20.0))
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
                header.col(|ui| {
                    ui.strong("");
                });
            })
            .body(|mut body| {
                for (mod_id, game_mod) in data.game_mods.iter_mut() {
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            if ui.checkbox(&mut game_mod.enabled, "").changed() {
                                self.should_integrate.store(true, Ordering::Release);
                            };
                        });
                        row.col(|ui| {
                            ui.label(&game_mod.name);
                        });
                        row.col(|ui| {
                            ui.push_id(mod_id, |ui| {
                                // becasue ComboBox .chnaged doesn't seem to work
                                let prev_selected = game_mod.selected_version;

                                Self::show_version_select(ui, game_mod);

                                // this may look dumb but is what is needed
                                if prev_selected != game_mod.selected_version {
                                    self.should_integrate.store(true, Ordering::Release);
                                }
                            });
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
                        row.col(|ui| {
                            if ui.button("More Info").clicked() {
                                self.selected_mod_id = Some(mod_id.to_owned());
                            };
                        });
                    });
                }
            });
    }

    // this is just an associated function to avoid upsetting the borrow checker
    fn show_version_select(ui: &mut egui::Ui, game_mod: &mut GameMod) {
        egui::ComboBox::from_id_source(&game_mod.name)
            .selected_text(format!("{}", game_mod.selected_version))
            .width(112.0)
            .show_ui(ui, |ui| {
                // for when there is an Index file show force latest version, this to diecrtly indicate that there
                // is the possibility of an auto update vie an index file.
                if game_mod.download.is_some() {
                    let latest_version = game_mod.latest_version.unwrap();
                    ui.selectable_value(
                        &mut game_mod.selected_version,
                        SelectedVersion::Latest(latest_version),
                        format!("{}", SelectedVersion::Latest(latest_version)),
                    );
                }

                // add all other versions to the drop down
                for version in game_mod.versions.iter() {
                    // if the version is the latest version, set it as LatestIndirect so that if there is an upgrade it will
                    // automatically be upgraded. This is under the assumption that if the user now has the latest version,
                    // that they probably also want to have the latest in the future.
                    let is_latest = *version.0
                        == game_mod
                            .latest_version
                            .unwrap_or_else(|| Version::new(0, 0, 0));

                    let show_version = if is_latest {
                        SelectedVersion::LatestIndirect(Some(*version.0))
                    } else {
                        SelectedVersion::Specific(*version.0)
                    };

                    ui.selectable_value(
                        &mut game_mod.selected_version,
                        show_version,
                        format!("{}", show_version),
                    );
                }
            });
    }

    fn show_bottom(&self, ui: &mut egui::Ui) {
        let data = self.data.lock();

        match self.selected_mod_id {
            Some(ref mod_id) => {
                let game_mod = data.game_mods.get(mod_id).unwrap();

                // ui.horizontal(|ui| {
                //     ui.label("Name:");
                ui.heading(&game_mod.name);
                //});

                ui.label(format!("Mod Id: {}", mod_id));
                ui.label(format!(
                    "Desciption: {}",
                    game_mod.description.as_ref().unwrap_or(&"None".to_owned())
                ));
                ui.label(format!("Sync: {}", game_mod.sync));
                //TODO: size
                ui.horizontal(|ui| {
                    ui.label("Website:");
                    match game_mod.homepage {
                        Some(ref url) => ui.hyperlink(url.as_str()),
                        None => ui.label("None"),
                    }
                });

                ui.label(egui::RichText::new("").size(5.0));
                ui.label(egui::RichText::new("Press DEL to remove this mod.").size(12.0));
            }
            None => {
                ui.label("Drop a .pak file onto this window to install the mod.");
                ui.label("To enable/disable mods click the checkbox to the left of the mod name.");
                ui.label("Then press \"Play\" ro start the game with mods.");
                ui.label(egui::RichText::new("").size(5.0));

                ui.label("Click on a mod to see more info.");
                ui.label(egui::RichText::new("").size(5.0));

                if cfg!(debug_assertions) {
                    egui::warn_if_debug_build(ui);
                    ui.label("Mod/game install folders.");
                    ui.label(match data.paks_path {
                        Some(ref path) => path.to_str().unwrap(),
                        None => "No paks path",
                    });
                    ui.label(match data.game_install_path {
                        Some(ref path) => path.to_str().unwrap(),
                        None => "No install path",
                    });
                }
            }
        }
    }

    fn show_footer(&self, ui: &mut egui::Ui) {
        let mut data = self.data.lock();

        StripBuilder::new(ui)
            .size(Size::relative(0.8))
            .size(Size::remainder())
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    if ui
                        .checkbox(
                            &mut data.refuse_mismatched_connections,
                            "Refuse mismatched connections",
                        )
                        .changed()
                    {
                        self.should_integrate.store(true, Ordering::Release);
                    };

                    ui.label(format!(
                        "Time since last integration {}s",
                        if self.working.load(Ordering::Acquire) {
                            0
                        } else {
                            self.last_integration_time.lock().elapsed().as_secs()
                        }
                    ));
                });

                strip.cell(|ui| {
                    ui.style_mut().spacing.button_padding = egui::vec2(9.0, 6.0);
                    ui.style_mut()
                        .text_styles
                        .get_mut(&egui::TextStyle::Button)
                        .unwrap()
                        .size = 16.0;

                    let layout = egui::Layout::from_main_dir_and_cross_align(
                        egui::Direction::RightToLeft,
                        egui::Align::Center,
                    );
                    ui.with_layout(layout, |ui| {
                        if ui.button("Play").clicked() {
                            let install_manager = data.get_install_manager();
                            if let Some(install_manager) = install_manager {
                                match install_manager.launch_game() {
                                    Ok(_) => {}
                                    Err(warn) => data.warnings.push(warn),
                                };
                            }
                        }
                    });
                });
            });

        // if ui
        //     .button(if !self.should_exit.load(Ordering::Acquire) {
        //         "Quit"
        //     } else {
        //         "Exiting..."
        //     })
        //     .clicked()
        // {
        //     frame.quit();
        // }
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
        #[allow(clippy::format_push_string)]
        // Preview hovering files
        if !ctx.input().raw.hovered_files.is_empty() {
            use egui::*;

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

        // Collect dropped files
        for dropped_file in ctx.input().raw.dropped_files.iter() {
            debug!("Dropped file: {:?}", dropped_file.path);

            self.data
                .lock()
                .files_to_process
                .push(dropped_file.path.as_ref().unwrap().to_owned());

            self.working.store(true, Ordering::Release);
        }
    }
}
