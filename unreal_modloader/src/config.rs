pub trait DummyIntegratorConfig: std::marker::Send {
    fn dummy(&self) -> String;
}

pub trait GameConfig<C>: std::marker::Send
where
    C: DummyIntegratorConfig,
{
    fn get_integrator_config(&self) -> &C;
    fn get_game_name(&self) -> String;
    fn get_app_id(&self) -> u32;
    fn get_window_title(&self) -> String;
}
