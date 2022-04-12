use unreal_modloader::config::DummyIntegratorConfig;

pub struct AstroIntegratorConfig;

impl DummyIntegratorConfig for AstroIntegratorConfig {
    fn dummy(&self) -> String {
        "dummy".to_string()
    }
}
