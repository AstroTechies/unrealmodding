use std::path::PathBuf;

use unreal_modintegrator::IntegratorConfig;

use crate::version::GameBuild;

pub trait GameConfig<'a, C, T, E: std::error::Error>: std::marker::Send
where
    C: IntegratorConfig<'a, T, E>,
{
    fn get_integrator_config(&self) -> &C;
    fn get_app_id(&self) -> u32;
    fn get_window_title(&self) -> String;
    fn get_game_build(&self, install_path: &PathBuf) -> Option<GameBuild>;
}
