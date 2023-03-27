use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use unreal_asset::engine_version::EngineVersion;
use unreal_mod_integrator::{HandlerFn, IntegratorConfig};
use unreal_pak::{PakMemory, PakReader};

pub struct Config;

#[allow(clippy::ptr_arg)]
fn handle_linked_actor_components(
    _data: &(),
    _integrated_pak: &mut PakMemory,
    _game_paks: &mut Vec<PakReader<File>>,
    _mod_paks: &mut Vec<PakReader<File>>,
    actors: &Vec<serde_json::Value>,
) -> Result<(), io::Error> {
    println!("Example linked actors: {actors:?}");
    Ok(())
}

impl<'data> IntegratorConfig<'data, (), io::Error> for Config {
    fn get_data(&self) -> &'data () {
        &()
    }

    fn get_handlers(&self) -> std::collections::HashMap<String, Box<HandlerFn<(), io::Error>>> {
        let mut handlers: std::collections::HashMap<String, Box<HandlerFn<(), io::Error>>> =
            HashMap::new();

        handlers.insert(
            String::from("linked_persistent_actors"),
            Box::new(handle_linked_actor_components),
        );

        handlers
    }

    // fn get_game_name(&self) -> String {
    //     String::from("ExampleGame")
    // }
    const GAME_NAME: &'static str = "ExampleGame";

    // fn get_integrator_version(&self) -> String {
    //     String::from("0.1.0")
    // }
    const INTEGRATOR_VERSION: &'static str = "0.1.0";

    // fn get_engine_version(&self) -> i32 {
    //     VER_UE4_23
    // }
    const ENGINE_VERSION: EngineVersion = EngineVersion::VER_UE4_23;

    fn get_baked_mods(&self) -> Vec<unreal_mod_integrator::IntegratorMod<io::Error>> {
        Vec::new()
    }
}

fn main() {
    let config = Config;
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        println!("Invalid args. Expected {{game_path}} {{mods_path}}");
    }

    let game_path = args[0].clone();
    let mods_path = args[1].clone();
    unreal_mod_integrator::integrate_mods(
        &config,
        &[],
        &PathBuf::from(&mods_path),
        &PathBuf::from(&game_path),
        true,
    )
    .unwrap();
}
