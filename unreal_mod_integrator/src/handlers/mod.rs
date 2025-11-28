use std::fs::File;
use std::io::BufReader;

use unreal_pak::{PakMemory, PakReader};

use crate::Error;

#[cfg(feature = "ue4_23")]
mod ue4_23;

#[cfg(feature = "ue4_27")]
mod ue4_27;

#[allow(unused_variables)]
#[allow(clippy::ptr_arg)]
pub fn handle_persistent_actors(
    game_name: &'static str,
    map_paths: &[&str],
    integrated_pak: &mut PakMemory,
    game_paks: &mut Vec<PakReader<BufReader<File>>>,
    mod_paks: &mut Vec<PakReader<BufReader<File>>>,
    persistent_actor_arrays: &Vec<serde_json::Value>,
) -> Result<(), Error> {
    #[cfg(feature = "ue4_23")]
    ue4_23::persistent_actors::handle_persistent_actors(
        game_name,
        map_paths,
        integrated_pak,
        game_paks,
        mod_paks,
        persistent_actor_arrays,
    )?;
    #[cfg(feature = "ue4_27")]
    ue4_27::persistent_actors::handle_persistent_actors(
        game_name,
        map_paths,
        integrated_pak,
        game_paks,
        mod_paks,
        persistent_actor_arrays,
    )?;

    Ok(())
}
