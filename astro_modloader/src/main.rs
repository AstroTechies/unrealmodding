#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use unreal_modintegrator::IntegratorConfig;
use unreal_modloader::config::GameConfig;

mod astro_integrator;
use astro_integrator::AstroIntegratorConfig;

struct AstroGameConfig;

impl<T, E: std::error::Error> GameConfig<'static, AstroIntegratorConfig, T, E> for AstroGameConfig
where
    AstroIntegratorConfig: IntegratorConfig<'static, T, E>,
{
    fn get_integrator_config(&self) -> &astro_integrator::AstroIntegratorConfig {
        &astro_integrator::AstroIntegratorConfig
    }

    fn get_game_name(&self) -> String {
        "Astro".to_string()
    }

    fn get_app_id(&self) -> u32 {
        361420
    }

    fn get_window_title(&self) -> String {
        "Astroneer Modloader".to_string()
    }
}

fn main() {
    println!("Astroneer Modloader");

    let config = AstroGameConfig;

    unreal_modloader::run(config);
}
