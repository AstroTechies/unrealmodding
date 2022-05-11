use std::path::Path;

use unreal_modintegrator::IntegratorConfig;

use crate::version::GameBuild;

pub trait GameConfig<'a, C, T, E: std::error::Error>: std::marker::Send
where
    C: IntegratorConfig<'a, T, E>,
{
    fn get_integrator_config(&self) -> &C;
    fn get_game_build(&self, install_path: &Path) -> Option<GameBuild>;

    const APP_ID: u32;
    const WINDOW_TITLE: &'static str;
}
