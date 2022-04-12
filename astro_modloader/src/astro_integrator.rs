use std::{collections::HashMap, io};

use unreal_asset::ue4version::VER_UE4_23;
use unreal_modintegrator::IntegratorConfig;
use unreal_pak::PakFile;

pub struct AstroIntegratorConfig;

fn handle_persistent_actors(
    _data: &(),
    _integrated_pak: &mut PakFile,
    _game_paks: &mut Vec<PakFile>,
    actors: Vec<&serde_json::Value>,
) -> Result<(), io::Error> {
    println!("{:?}", actors);
    Ok(())
}

impl<'data> IntegratorConfig<'data, (), io::Error> for AstroIntegratorConfig {
    fn get_data(&self) -> &'data () {
        &()
    }

    fn get_handlers(
        &self,
    ) -> std::collections::HashMap<
        String,
        Box<
            dyn FnMut(
                &(),
                &mut unreal_pak::PakFile,
                &mut Vec<unreal_pak::PakFile>,
                Vec<&serde_json::Value>,
            ) -> Result<(), io::Error>,
        >,
    > {
        let mut handlers: std::collections::HashMap<
            String,
            Box<
                dyn FnMut(
                    &(),
                    &mut unreal_pak::PakFile,
                    &mut Vec<unreal_pak::PakFile>,
                    Vec<&serde_json::Value>,
                ) -> Result<(), io::Error>,
            >,
        > = HashMap::new();

        handlers.insert(
            String::from("persistent_actors"),
            Box::new(handle_persistent_actors),
        );

        handlers
    }

    fn get_integrator_version(&self) -> String {
        String::from("0.1.0")
    }

    fn get_refuse_mismatched_connections(&self) -> bool {
        true
    }

    fn get_engine_version(&self) -> i32 {
        VER_UE4_23
    }
}
