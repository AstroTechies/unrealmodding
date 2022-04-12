use unreal_modloader;

mod astro_integrator;

struct AstroGameConfig;

impl unreal_modloader::config::GameConfig<astro_integrator::AstroIntegratorConfig>
    for AstroGameConfig
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
