use std::cell::Cell;
use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    mpsc::Sender,
    Arc,
};
use std::time::Instant;

use eframe::{
    egui::{self, Button, ProgressBar, Sense, Widget},
    emath::Align,
    App, Frame,
};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use log::info;
use parking_lot::Mutex;
use semver::Version;

use crate::egui::ViewportCommand;
use crate::background_work::BackgroundThreadMessage;
use crate::error::{ModLoaderError, ModLoaderWarning};
use crate::game_mod::{GameMod, SelectedVersion};
use crate::mod_processing::dependencies::DependencyGraph;
use crate::profile::{Profile, ProfileMod};
use crate::update_info::UpdateInfo;
use crate::{FileToProcess, ModLoaderAppData};

pub(crate) struct ModLoaderApp {
    pub data: Arc<Mutex<ModLoaderAppData>>,
    pub background_tx: Sender<BackgroundThreadMessage>,

    pub window_title: String,
    pub modloader_version: &'static str,

    pub ready_exit: Arc<AtomicBool>,
    pub working: Arc<AtomicBool>,
    pub last_integration_time: Arc<Mutex<Instant>>,
    pub updating: Cell<bool>,

    pub newer_update: Arc<Mutex<Option<UpdateInfo>>>,
    pub update_progress: Arc<AtomicI32>,

    pub platform_selector_open: bool,
    pub untrusted_mods_open: bool,
    pub selected_mod_id: Option<String>,
    pub profile_manager_open: Cell<bool>,

    pub about_text: String,
    pub about_open: Cell<bool>,

    markdown_cache: CommonMarkCache,
}

impl ModLoaderApp {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data: Arc<Mutex<ModLoaderAppData>>,
        background_tx: Sender<BackgroundThreadMessage>,
        window_title: String,
        modloader_version: &'static str,
        ready_exit: Arc<AtomicBool>,
        working: Arc<AtomicBool>,
        last_integration_time: Arc<Mutex<Instant>>,
        newer_update: Arc<Mutex<Option<UpdateInfo>>>,
        update_progress: Arc<AtomicI32>,
        about_text: String,
    ) -> Self {
        ModLoaderApp {
            data,
            background_tx,

            window_title,
            modloader_version,

            ready_exit,
            working,
            last_integration_time,
            updating: Cell::new(false),

            newer_update,
            update_progress,

            platform_selector_open: false,
            untrusted_mods_open: false,
            selected_mod_id: None,
            profile_manager_open: Cell::new(false),

            about_text,
            about_open: Cell::new(false),

            markdown_cache: CommonMarkCache::default(),
        }
    }
}

impl App for ModLoaderApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Main UI

        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(22.0))
                .size(Size::relative(0.45))
                .size(Size::remainder())
                .size(Size::exact(14.0))
                .size(Size::exact(2.0))
                .size(Size::exact(45.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        self.show_header(ui);
                    });
                    // seperators only look good if at the start of a cell, not the end of the previous one.
                    strip.cell(|ui| {
                        ui.separator();
                        self.show_table(ui);
                    });
                    strip.cell(|ui| {
                        ui.separator();
                        self.show_bottom(ui);
                    });

                    strip.cell(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("About").clicked() {
                                self.about_open.set(true);
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                ui.label(format!("Version: {}", self.modloader_version));
                            });
                        });
                    });

                    // spacing
                    strip.cell(|_| {});

                    strip.cell(|ui| {
                        ui.separator();
                        self.show_footer(ui);
                    });
                });
        });

        // "popup" windows

        let mut darken_background = false;

        let mut update_cancelled = false;
        let has_newer_update = self.newer_update.lock().is_some();
        if has_newer_update {
            self.show_update_window(ctx, &mut update_cancelled);
            darken_background = true;
        }

        if update_cancelled {
            *self.newer_update.lock() = None;
        }

        let mut data = self.data.lock();
        if let Some(error) = &data.error {
            self.show_error(ctx, frame, error);
            darken_background = true;
        } else if !data.warnings.is_empty() {
            self.show_warnings(ctx, &mut data.warnings);
            darken_background = true;
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
                            false => {
                                Button::new(format!("{platform} (not found)")).sense(Sense::hover())
                            }
                        };

                        if ui.add(button).clicked() {
                            let _ = data.set_game_platform(&platform);
                            self.platform_selector_open = false;
                            let _ = self
                                .background_tx
                                .send(BackgroundThreadMessage::integrate());
                            ctx.request_repaint();
                        }
                    }
                });
            darken_background = true;
        }

        if self.profile_manager_open.get() {
            self.show_profile_manager(ctx, &mut data);
            darken_background = true;
        }

        drop(data);

        if self.untrusted_mods_open {
            self.show_untrusted_mods(ctx);
            darken_background = true;
        }

        if self.about_open.get() {
            self.show_about(ctx);
            darken_background = true;
        }

        // Keyboard shortcuts

        // esc show default bottom text
        if ctx.input(|e| e.key_pressed(egui::Key::Escape)) {
            self.selected_mod_id = None;
        }

        // delete to remove a mod
        if ctx.input(|e| e.key_pressed(egui::Key::Delete)) {
            if let Some(ref id) = self.selected_mod_id {
                let _ = self
                    .background_tx
                    .send(BackgroundThreadMessage::RemoveMod(id.clone()));
                self.selected_mod_id = None;
                let _ = self
                    .background_tx
                    .send(BackgroundThreadMessage::integrate());
            }
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            let _ = self.background_tx.send(BackgroundThreadMessage::Exit);

            if self.ready_exit.load(Ordering::Acquire) {
                info!("Exiting...");
            }
    
            self.ready_exit.load(Ordering::Acquire);
        }

        if darken_background {
            self.darken_background(ctx);
        }

        self.detect_files_being_dropped(ctx);

        // We need to keep the paint loop constantly running when shutting down,
        // otherwise the background thread might be done, but the paint loop is
        // in idle becasue there is no user input.
        // Or keep it running while the background thread is actively working.
        // Or while the last integration was not long ago.
        if self.working.load(Ordering::Acquire)
            || self.last_integration_time.lock().elapsed().as_secs() < 5
        {
            ctx.request_repaint();
        }

        // when background thread is ready to exit kill app by ending main thread
        if self.ready_exit.load(Ordering::Acquire) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

impl ModLoaderApp {
    // Main UI parts

    fn show_header(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::relative(0.5))
            .size(Size::remainder())
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    self.show_title(ui);
                });
                strip.cell(|ui| {
                    self.show_header_right(ui);
                });
            });
    }

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
            ui.heading(format!("{title} - Working..."));
        }
    }

    fn show_header_right(&mut self, ui: &mut egui::Ui) {
        let data = self.data.lock();

        let current_platform = format!(
            "Platform: {}",
            match data.selected_game_platform {
                Some(ref platform) => platform.to_string(),
                None => "None".to_owned(),
            },
        );

        ui.with_layout(ui.layout().with_cross_align(Align::Max), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Profiles").clicked() {
                    self.profile_manager_open.set(true);
                }
                if ui.button("Change platform").clicked() {
                    self.platform_selector_open = true;
                }
                ui.label(current_platform);
            });
        });
    }

    fn show_table(&mut self, ui: &mut egui::Ui) {
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::initial(40.0).at_least(40.0).at_most(40.0))
            .column(Column::initial(170.0).at_least(20.0))
            .column(Column::initial(115.0).at_least(115.0).at_most(115.0))
            .column(Column::initial(115.0).at_least(20.0).at_most(115.0))
            .column(Column::initial(80.0).at_least(20.0))
            .column(Column::remainder().at_least(20.0))
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
                let mut data = self.data.lock();

                // ugly hack to bypass borrow checker
                // this is safe because we are getting mut references to different struct fields
                let dependency_graph = &data.dependency_graph as *const Option<DependencyGraph>;
                let warnings = &mut data.warnings as *mut Vec<ModLoaderWarning>;

                for (mod_id, game_mod) in data.game_mods.iter_mut() {
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            if ui.checkbox(&mut game_mod.enabled, "").changed() {
                                if !game_mod.enabled {
                                    // ugly hack to bypass borrow checker
                                    if let Some(dependency_graph) = unsafe { &*dependency_graph } {
                                        let dependents =
                                            dependency_graph.find_mod_dependents(mod_id);
                                        if !dependents.is_empty() {
                                            unsafe { &mut *warnings }.push(
                                                ModLoaderWarning::referenced_by_other_mods(
                                                    mod_id.clone(),
                                                    dependents,
                                                ),
                                            )
                                        }
                                    }
                                }
                                let _ = self
                                    .background_tx
                                    .send(BackgroundThreadMessage::integrate());
                            };
                        });
                        row.col(|ui| {
                            ui.label(&game_mod.name);
                        });
                        row.col(|ui| {
                            ui.push_id(mod_id, |ui| {
                                // becasue ComboBox .chnaged doesn't seem to work
                                let prev_selected = game_mod.selected_version.clone();

                                Self::show_version_select(ui, game_mod);

                                // this may look dumb but is what is needed
                                if prev_selected != game_mod.selected_version {
                                    let _ = self
                                        .background_tx
                                        .send(BackgroundThreadMessage::integrate());
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
        egui::ComboBox::from_id_salt(&game_mod.name)
            .selected_text(format!("{}", game_mod.selected_version))
            .width(112.0)
            .show_ui(ui, |ui| {
                // for when there is an Index file show force latest version, this to diecrtly indicate that there
                // is the possibility of an auto update vie an index file.
                if game_mod.download.is_some() && game_mod.download.as_ref().unwrap().download_mode == unreal_mod_metadata::DownloadMode::IndexFile {
                    let latest_version = game_mod.latest_version.clone().unwrap();
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
                    let is_latest = *version.0
                        == game_mod
                            .latest_version
                            .clone()
                            .unwrap_or_else(|| Version::new(0, 0, 0));

                    let show_version = if is_latest {
                        SelectedVersion::LatestIndirect(Some(version.0.clone()))
                    } else {
                        SelectedVersion::Specific(version.0.clone())
                    };

                    ui.selectable_value(
                        &mut game_mod.selected_version,
                        show_version.clone(),
                        format!("{show_version}"),
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

                ui.label(format!("Mod Id: {mod_id}"));
                ui.label(format!(
                    "Desciption: {}",
                    game_mod.description.as_ref().unwrap_or(&"None".to_owned())
                ));
                ui.label(format!("Sync: {}", game_mod.sync));
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
                ui.label("Then press \"Play\" to start the game with mods.");
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

    fn show_footer(&mut self, ui: &mut egui::Ui) {
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
                        let _ = self
                            .background_tx
                            .send(BackgroundThreadMessage::integrate());
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
                        let button = match data.failed || self.working.load(Ordering::Acquire) {
                            true => Button::new("Play").sense(Sense::hover()),
                            false => Button::new("Play"),
                        };
                        if ui.add(button).clicked() {
                            if !data.untrusted_mods.is_empty() {
                                self.untrusted_mods_open = true;
                            } else {
                                let _ =
                                    self.background_tx.send(BackgroundThreadMessage::LaunchGame);
                            }
                        }
                    });
                });
            });
    }

    fn darken_background(&mut self, ctx: &egui::Context) {
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::PanelResizeLine,
            egui::Id::new("panel_darken"),
        ));

        let screen_rect = ctx.input(|e| e.screen_rect());
        painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(192));
    }

    // from https://github.com/emilk/egui/blob/master/examples/file_dialog/src/main.rs
    fn detect_files_being_dropped(&mut self, ctx: &egui::Context) {
        #[allow(clippy::format_push_string)]
        // Preview hovering files
        if !ctx.input(|e| e.raw.hovered_files.is_empty()) {
            use egui::*;

            let text = ctx.input(|i| {
                use std::fmt::Write as _;

                let mut text = "Dropping files:\n".to_owned();
                for file in &i.raw.hovered_files {
                    if let Some(path) = &file.path {
                        write!(text, "\n{}", path.display()).ok();
                    } else if !file.mime.is_empty() {
                        write!(text, "\n{}", file.mime).ok();
                    } else {
                        text += "\n???";
                    }
                }
                text
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.input(|e| e.screen_rect());
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
        let files_to_import = ctx.input(|e| {
            e.raw
                .dropped_files
                .iter()
                .map(|e| FileToProcess::new(e.path.as_ref().unwrap().to_owned(), true))
                .collect::<Vec<_>>()
        });

        if !files_to_import.is_empty() {
            let _ = self
                .background_tx
                .send(BackgroundThreadMessage::Import(files_to_import));
            let _ = self
                .background_tx
                .send(BackgroundThreadMessage::integrate());
        }
    }

    // "popup" windows

    fn show_update_window(&mut self, ctx: &egui::Context, update_cancelled: &mut bool) {
        let newer_update = self.newer_update.lock();
        let newer_update = newer_update.as_ref().unwrap();
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
                            ui.heading("Changelog");
                            CommonMarkViewer::new().show_scrollable(
                                "update_viewer",
                                ui,
                                &mut self.markdown_cache,
                                &newer_update.changelog,
                            );
                        });

                        strip.cell(|ui| {
                            if self.updating.get() {
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
                                                // todo: error
                                                let _ = self
                                                    .background_tx
                                                    .send(BackgroundThreadMessage::UpdateApp);
                                                self.updating.set(true);
                                            }
                                        });

                                        strip.cell(|ui| {
                                            if ui.button("Cancel").clicked() {
                                                *update_cancelled = true;
                                            }
                                        });
                                    });
                            });
                        });
                    });
            });
    }

    #[allow(unused_variables)]
    fn show_error(&self, ctx: &egui::Context, frame: &mut Frame, error: &ModLoaderError) {
        egui::Window::new("Critical Error")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
            .fixed_size((600.0, 400.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(10.0, 25.0);

                    ui.label(format!("{error}"));
                });

                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.style_mut().spacing.button_padding = egui::vec2(6.0, 6.0);
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });
            });
    }

    fn show_warnings(&self, ctx: &egui::Context, warnings: &mut Vec<ModLoaderWarning>) {
        egui::Window::new("Warning")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
            .fixed_size((600.0, 400.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for warning in warnings.iter() {
                        ui.label(format!("{warning}"));
                    }

                    ui.label("");
                    ui.label("See modloader_log.txt for more details.");
                    ui.label("");
                });

                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.style_mut().spacing.button_padding = egui::vec2(6.0, 6.0);
                    if ui.button("Ok").clicked() {
                        warnings.clear();
                    }
                });
            });
    }

    fn show_profile_manager(&self, ctx: &egui::Context, data: &mut ModLoaderAppData) {
        let mut changed = false;

        egui::Window::new("Profiles")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
            .fixed_size((445.0, 400.0))
            .show(ctx, |ui| {
                StripBuilder::new(ui)
                    .size(Size::remainder())
                    .size(Size::exact(40.0))
                    .vertical(|mut strip| {
                        // Profile list
                        strip.cell(|ui| {
                            TableBuilder::new(ui)
                                .striped(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::exact(280.0))
                                .column(Column::auto())
                                .resizable(false)
                                .header(20.0, |mut header| {
                                    header.col(|ui| {
                                        ui.strong("Name");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Actions");
                                    });
                                })
                                .body(|mut body| {
                                    let mut remove = None;
                                    for (i, profile) in data.profiles.iter_mut().enumerate() {
                                        body.row(18.0, |mut row| {
                                            row.col(|ui| {
                                                if ui
                                                    .text_edit_singleline(&mut profile.name)
                                                    .changed()
                                                {
                                                    changed = true;
                                                }
                                            });
                                            row.col(|ui| {
                                                if ui.button("Save").clicked() {
                                                    profile.mods = data
                                                        .game_mods
                                                        .iter()
                                                        .filter(|(_, game_mod)| game_mod.enabled)
                                                        .map(|(mod_id, game_mod)| {
                                                            (
                                                                mod_id.clone(),
                                                                ProfileMod {
                                                                    force_latest: game_mod
                                                                        .selected_version
                                                                        .is_latest(),
                                                                    version: game_mod
                                                                        .selected_version
                                                                        .clone()
                                                                        .unwrap()
                                                                        .to_string(),
                                                                    ..Default::default()
                                                                },
                                                            )
                                                        })
                                                        .collect();
                                                    changed = true;
                                                };

                                                if ui.button("Load").clicked() {
                                                    for (mod_id, game_mod) in
                                                        data.game_mods.iter_mut()
                                                    {
                                                        if let Some(_profile_entry) =
                                                            profile.mods.get(mod_id)
                                                        {
                                                            game_mod.enabled = true;

                                                            // TODO load version
                                                        } else {
                                                            game_mod.enabled = false;
                                                        }
                                                    }

                                                    let _ = self
                                                        .background_tx
                                                        .send(BackgroundThreadMessage::integrate());
                                                }

                                                if ui.button("Delete").clicked() {
                                                    remove = Some(i);
                                                }
                                            });
                                        });
                                    }
                                    if let Some(i) = remove {
                                        data.profiles.remove(i);
                                        changed = true;
                                    }
                                });
                        });

                        // Footer
                        strip.cell(|ui| {
                            ui.separator();

                            // big buttons
                            ui.style_mut().spacing.button_padding = egui::vec2(6.0, 6.0);

                            StripBuilder::new(ui)
                                .size(Size::relative(0.5))
                                .size(Size::remainder())
                                .horizontal(|mut strip| {
                                    strip.cell(|ui| {
                                        ui.with_layout(
                                            egui::Layout::left_to_right(egui::Align::Min),
                                            |ui| {
                                                if ui.button("New").clicked() {
                                                    data.profiles.push(Profile {
                                                        name: "New Profile".to_owned(),
                                                        ..Default::default()
                                                    });
                                                    changed = true;
                                                }
                                            },
                                        );
                                    });
                                    strip.cell(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Min),
                                            |ui| {
                                                if ui.button("Close").clicked() {
                                                    self.profile_manager_open.set(false);
                                                }
                                            },
                                        );
                                    });
                                });
                        })
                    });
            });

        if changed {
            let _ = self
                .background_tx
                .send(BackgroundThreadMessage::WriteConfig);
        }
    }

    fn show_untrusted_mods(&mut self, ctx: &egui::Context) {
        let mut data = self.data.lock();

        egui::Window::new("Untrusted Mods")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
            .fixed_size((600.0, 400.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            "These mods may contain code that can harm your computer, trust them?",
                        );
                    });

                    ui.add_space(8.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for untrusted_mod in &data.untrusted_mods {
                            ui.label(format!(
                                "{} version {}",
                                untrusted_mod.name, untrusted_mod.version
                            ));
                        }
                    });

                    ui.add_space(8.0);

                    ui.separator();

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.style_mut().spacing.button_padding = egui::vec2(6.0, 6.0);

                        if ui.button("Trust").clicked() {
                            let untrusted_mods = data
                                .untrusted_mods
                                .iter()
                                .map(|e| e.hash.clone())
                                .collect::<Vec<_>>();

                            data.trusted_mods.extend(untrusted_mods);
                            data.untrusted_mods.clear();

                            let _ = self
                                .background_tx
                                .send(BackgroundThreadMessage::WriteConfig);

                            let _ = self.background_tx.send(BackgroundThreadMessage::LaunchGame);

                            self.untrusted_mods_open = false;
                        }

                        if ui.button("Cancel").clicked() {
                            self.untrusted_mods_open = false;
                        }
                    });
                });
            });
    }

    fn show_about(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .resizable(true)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, (0.0, 50.0))
            .default_size((600.0, 400.0))
            .vscroll(false)
            .show(ctx, |ui| {
                egui::TopBottomPanel::bottom("bottom_panel")
                    .resizable(false)
                    .min_height(0.0)
                    .show_inside(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add_space(8.0);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                ui.style_mut().spacing.button_padding = egui::vec2(6.0, 6.0);
                                if ui.button("Close").clicked() {
                                    self.about_open.set(false);
                                }
                            });
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    CommonMarkViewer::new().show_scrollable(
                        "viewer",
                        ui,
                        &mut self.markdown_cache,
                        &self.about_text,
                    );
                });
            });
    }
}
