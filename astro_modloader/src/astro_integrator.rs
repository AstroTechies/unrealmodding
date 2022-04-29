use std::{
    collections::HashMap,
    io::{self},
};

use unreal_asset::ue4version::VER_UE4_23;
use unreal_modintegrator::IntegratorConfig;

use crate::handlers::{
    item_list_entries, linked_actor_components, mission_trailheads, persistent_actors,
};

pub struct AstroIntegratorConfig;

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
            Box::new(persistent_actors::handle_persistent_actors),
        );

        handlers.insert(
            String::from("mission_trailheads"),
            Box::new(mission_trailheads::handle_mission_trailheads),
        );

        handlers.insert(
            String::from("linked_actor_components"),
            Box::new(linked_actor_components::handle_linked_actor_components),
        );

        handlers.insert(
            String::from("item_list_entries"),
            Box::new(item_list_entries::handle_item_list_entries),
        );

        handlers
    }

    fn get_game_name(&self) -> String {
        "Astro".to_string()
    }

    fn get_integrator_version(&self) -> String {
        String::from("0.1.0")
    }

    fn get_engine_version(&self) -> i32 {
        VER_UE4_23
    }
}
