use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicI32},
    Arc,
};
use std::thread;
use std::time::Instant;

use background_work::BackgroundThreadData;
use config::InstallManager;
use eframe::egui;
use log::error;

use parking_lot::Mutex;

use unreal_modintegrator::IntegratorConfig;

mod app;
mod background_work;
pub mod config;
pub mod error;
pub(crate) mod game_mod;
pub mod game_path_helpers;
pub mod game_platform_managers;
mod mod_config;
mod mod_processing;
pub mod update_info;
pub mod version;

use error::{ModLoaderError, ModLoaderWarning};
use game_mod::GameMod;
use mod_config::write_config;
use version::GameBuild;

pub use unreal_asset;
pub use unreal_modintegrator;
pub use unreal_modmetadata;
pub use unreal_pak;

use crate::mod_processing::dependencies::DependencyGraph;

#[derive(Debug, Clone)]
pub(crate) struct FileToProcess {
    pub path: PathBuf,
    pub newly_added: bool,
}

impl FileToProcess {
    pub fn new(path: PathBuf, newly_added: bool) -> Self {
        FileToProcess { path, newly_added }
    }
}

#[derive(Debug)]
pub(crate) struct ModLoaderAppData {
    /// %LocalAppData%\[GameName]\Saved\Mods
    pub mods_path: Option<PathBuf>,
    /// %LocalAppData%\[GameName]\Saved\Paks
    pub paks_path: Option<PathBuf>,
    /// game install path
    pub game_install_path: Option<PathBuf>,

    pub game_build: Option<GameBuild>,
    pub refuse_mismatched_connections: bool,
    pub files_to_process: Vec<FileToProcess>,

    pub game_mods: BTreeMap<String, GameMod>,

    pub error: Option<ModLoaderError>,
    pub warnings: Vec<ModLoaderWarning>,

    pub failed: bool,

    pub dependency_graph: Option<DependencyGraph>,

    /// install managers
    pub(crate) install_managers: BTreeMap<&'static str, Box<dyn InstallManager>>,
    pub(crate) selected_game_platform: Option<String>,
}

impl ModLoaderAppData {
    pub fn set_game_platform(&mut self, platform: &str) -> bool {
        let manager = self.install_managers.get(platform);
        if let Some(manager) = manager {
            self.game_install_path = manager.get_game_install_path();
            self.game_build = manager.get_game_build();
            self.paks_path = manager.get_paks_path();

            self.selected_game_platform = Some(platform.to_string());

            write_config(self);
            return true;
        }
        false
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_install_manager(&self) -> Option<&Box<dyn InstallManager>> {
        if let Some(platform) = &self.selected_game_platform {
            return self.install_managers.get(&platform.as_str());
        }

        None
    }
}

pub fn run<'a, C, D, T: 'a, E: 'static + std::error::Error + Send>(config: C)
where
    D: 'static + IntegratorConfig<'a, T, E>,
    C: 'static + config::GameConfig<'a, D, T, E>,
{
    let data = Arc::new(Mutex::new(ModLoaderAppData {
        mods_path: None,
        paks_path: None,
        game_install_path: None,

        game_build: None,
        refuse_mismatched_connections: true,
        game_mods: BTreeMap::new(),
        files_to_process: Vec::new(),

        error: None,
        warnings: Vec::new(),
        install_managers: config.get_install_managers(),
        selected_game_platform: None,
        failed: false,
        dependency_graph: None,
    }));

    let icon_data = config.get_icon();

    let should_exit = Arc::new(AtomicBool::new(false));
    let ready_exit = Arc::new(AtomicBool::new(false));
    let should_integrate = Arc::new(AtomicBool::new(true));
    let last_integration_time = Arc::new(Mutex::new(Instant::now()));
    let working = Arc::new(AtomicBool::new(true));

    let newer_update = Arc::new(Mutex::new(config.get_newer_update().ok().flatten()));

    let should_update = Arc::new(AtomicBool::new(false));
    let update_progress = Arc::new(AtomicI32::new(0));

    // instantiate the GUI app
    let app = app::ModLoaderApp {
        data: Arc::clone(&data),
        window_title: C::WINDOW_TITLE.to_owned(),

        should_exit: Arc::clone(&should_exit),
        ready_exit: Arc::clone(&ready_exit),

        should_integrate: Arc::clone(&should_integrate),
        last_integration_time: Arc::clone(&last_integration_time),

        working: Arc::clone(&working),

        platform_selector_open: false,
        selected_mod_id: None,
        newer_update: Arc::clone(&newer_update),
        should_update: Arc::clone(&should_update),
        update_progress: Arc::clone(&update_progress),

        modloader_version: C::CRATE_VERSION,
    };

    // spawn a background thread to handle long running tasks

    let background_thread_data = BackgroundThreadData {
        data,
        should_exit,
        ready_exit,
        should_integrate,
        last_integration_time,
        working,
        newer_update,
        should_update,
        update_progress,
    };

    thread::Builder::new()
        .name("background".to_string())
        .spawn(move || background_work::background_work(config, background_thread_data))
        .unwrap_or_else(|_| {
            error!("Failed to start background thread");
            panic!();
        });

    let icon_data = match icon_data {
        Some(data) => Some(eframe::IconData {
            rgba: data.data.to_vec(),
            width: data.width,
            height: data.height,
        }),
        None => None,
    };

    // run the GUI app
    eframe::run_native(
        app.window_title.clone().as_str(),
        eframe::NativeOptions {
            follow_system_theme: true,
            initial_window_size: Some(eframe::egui::vec2(623.0, 600.0)),
            icon_data,
            ..eframe::NativeOptions::default()
        },
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.iter_mut().for_each(|font| {
                font.1.tweak.scale = 1.15;
            });
            cc.egui_ctx.set_fonts(fonts);

            cc.egui_ctx.set_style(egui::Style::default());

            Box::new(app)
        }),
    );
}
