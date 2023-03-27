use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicI32},
    mpsc, Arc,
};
use std::thread;
use std::time::Instant;

use eframe::egui;
use log::error;
use parking_lot::Mutex;

use unreal_mod_integrator::IntegratorConfig;

mod app;
mod background_work;
pub mod config;
pub mod error;
pub(crate) mod game_mod;
pub mod game_path_helpers;
pub mod game_platform_managers;
mod mod_config;
mod mod_processing;
pub(crate) mod profile;
pub mod update_info;
pub mod version;

use background_work::{BackgroundThreadData, BackgroundThreadMessage};
use config::InstallManager;
use error::{ModLoaderError, ModLoaderWarning};
use game_mod::GameMod;
use mod_config::write_config;
use mod_processing::dependencies::DependencyGraph;
use version::GameBuild;

pub use unreal_asset;
#[cfg(feature = "cpp_loader")]
pub use unreal_cpp_bootstrapper;
pub use unreal_helpers;
pub use unreal_mod_integrator;
pub use unreal_mod_metadata;
pub use unreal_pak;

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

#[derive(Debug, Clone)]
pub(crate) struct UntrustedMod {
    pub name: String,
    pub version: String,
    pub hash: Vec<u8>,
}

impl UntrustedMod {
    pub fn new(name: String, version: String, hash: Vec<u8>) -> Self {
        UntrustedMod {
            name,
            version,
            hash,
        }
    }
}

#[derive(Debug, Default)]
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
    pub profiles: Vec<profile::Profile>,

    pub error: Option<ModLoaderError>,
    pub warnings: Vec<ModLoaderWarning>,

    pub failed: bool,

    pub dependency_graph: Option<DependencyGraph>,

    /// install managers
    pub install_managers: BTreeMap<&'static str, Box<dyn InstallManager>>,
    pub selected_game_platform: Option<String>,

    pub trusted_mods: Vec<Vec<u8>>,
    pub untrusted_mods: Vec<UntrustedMod>,

    #[cfg(feature = "cpp_loader")]
    pub(crate) cpp_loader_config: unreal_cpp_bootstrapper::config::GameSettings,
    #[cfg(feature = "cpp_loader")]
    pub(crate) cpp_loader_extract_path: Option<PathBuf>,
}

impl ModLoaderAppData {
    pub fn set_game_platform(&mut self, platform: &str) -> bool {
        let manager = self.install_managers.get(platform);
        if let Some(manager) = manager {
            self.game_install_path = manager.get_game_install_path();
            self.game_build = manager.get_game_build();
            self.paks_path = manager.get_paks_path();

            #[cfg(feature = "cpp_loader")]
            {
                self.cpp_loader_extract_path = manager.get_extract_path();
            }

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

pub fn run<'data, GC, IC, D: 'data, E: 'static + std::error::Error + Send>(config: GC)
where
    GC: 'static + config::GameConfig<'data, IC, D, E>,
    IC: 'static + IntegratorConfig<'data, D, E>,
{
    let data = Arc::new(Mutex::new(ModLoaderAppData {
        refuse_mismatched_connections: true,
        install_managers: config.get_install_managers(),
        #[cfg(feature = "cpp_loader")]
        cpp_loader_config: GC::get_cpp_loader_config(),
        ..Default::default()
    }));

    let icon_data = config.get_icon();

    let ready_exit = Arc::new(AtomicBool::new(false));
    let last_integration_time = Arc::new(Mutex::new(Instant::now()));
    let working = Arc::new(AtomicBool::new(true));

    let newer_update = Arc::new(Mutex::new(config.get_newer_update().ok().flatten()));

    let should_update = Arc::new(AtomicBool::new(false));
    let update_progress = Arc::new(AtomicI32::new(0));

    let (background_tx, background_rx) = mpsc::channel::<BackgroundThreadMessage>();

    // Only integrate if there is no update
    let has_newer_update = config
        .get_newer_update()
        .map(|e| e.is_some())
        .unwrap_or(false);

    if !has_newer_update {
        let _ = background_tx.send(BackgroundThreadMessage::integrate());
    }

    // instantiate the GUI app
    let app = app::ModLoaderApp::new(
        data.clone(),
        background_tx,
        GC::WINDOW_TITLE.to_owned(),
        GC::CRATE_VERSION,
        ready_exit.clone(),
        working.clone(),
        last_integration_time.clone(),
        newer_update.clone(),
        update_progress.clone(),
    );

    // spawn a background thread to handle long running tasks

    let background_thread_data = BackgroundThreadData {
        data,
        ready_exit,
        last_integration_time,
        working,
        newer_update,
        should_update,
        update_progress,
    };

    thread::Builder::new()
        .name("background".to_string())
        .spawn(move || {
            background_work::background_work(config, background_thread_data, background_rx)
        })
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
            initial_window_size: Some(eframe::egui::vec2(660.0, 600.0)),
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

pub const fn default_true() -> bool {
    true
}
