#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use unreal_modintegrator::IntegratorConfig;
use unreal_modloader::config::GameConfig;
use unreal_modloader::version::GameBuild;

mod astro_integrator;
use astro_integrator::AstroIntegratorConfig;

mod assets;
mod logging;

use log::info;

struct AstroGameConfig;

impl<T, E: std::error::Error> GameConfig<'static, AstroIntegratorConfig, T, E> for AstroGameConfig
where
    AstroIntegratorConfig: IntegratorConfig<'static, T, E>,
{
    fn get_integrator_config(&self) -> &astro_integrator::AstroIntegratorConfig {
        &astro_integrator::AstroIntegratorConfig
    }

    fn get_app_id(&self) -> u32 {
        361420
    }

    fn get_window_title(&self) -> String {
        "Astroneer Modloader".to_string()
    }

    fn get_game_build(&self, install_path: &PathBuf) -> Option<GameBuild> {
        let version_file_path = install_path.join("build.version");
        if !version_file_path.is_file() {
            info!("{:?} not found", version_file_path);
            return None;
        }

        let version_file = std::fs::read_to_string(&version_file_path).unwrap();
        let game_build_string = version_file.split(' ').next().unwrap().to_owned();

        GameBuild::try_from(&game_build_string).ok()
    }
}

fn main() {
    logging::init().unwrap();

    info!("Astroneer Modloader");

    let config = AstroGameConfig;

    unreal_modloader::run(config);
}
