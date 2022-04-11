pub trait DummyIntegratorConfig {
    fn dummy(&self) -> String;
}

pub trait GameConfig<C>
where
    C: DummyIntegratorConfig,
{
    fn get_integrator_config(&self) -> &C;
    fn get_game_name(&self) -> String;
    fn get_window_title(&self) -> String;
}
