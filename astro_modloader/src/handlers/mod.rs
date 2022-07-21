use lazy_static::lazy_static;

use regex::Regex;

pub(crate) mod item_list_entries;
pub(crate) mod linked_actor_components;
pub(crate) mod mission_trailheads;
pub mod persistent_actors;

lazy_static! {
    static ref GAME_REGEX: Regex = Regex::new(r"^/Game/").unwrap();
}

static MAP_PATHS: [&str; 3] = [
    "Astro/Content/Maps/Staging_T2.umap",
    "Astro/Content/Maps/Staging_T2_PackedPlanets_Switch.umap",
    //"Astro/Content/Maps/TutorialMoon_Prototype_v2.umap", // Tutorial not integrated for performance
    "Astro/Content/Maps/test/BasicSphereT2.umap",
];
