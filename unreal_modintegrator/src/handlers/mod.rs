use std::io;

use unreal_pak::PakFile;

#[cfg(feature = "ue4_23")]
mod ue4_23;

pub fn handle_persistent_actors(
    game_name: &'static str,
    map_paths: &[&str],
    integrated_pak: &mut PakFile,
    game_paks: &mut Vec<PakFile>,
    mod_paks: &mut Vec<PakFile>,
    persistent_actor_arrays: &Vec<serde_json::Value>,
) -> Result<(), io::Error> {
    #[cfg(feature = "ue4_23")]
    ue4_23::persistent_actors::handle_persistent_actors(
        game_name,
        map_paths,
        integrated_pak,
        game_paks,
        mod_paks,
        persistent_actor_arrays,
    )?;

    Ok(())
}
