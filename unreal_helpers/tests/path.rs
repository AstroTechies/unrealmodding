#![cfg(feature = "path")]

use unreal_helpers::game_to_absolute;

#[test]
fn test_game_to_absolute() {
    let game_name = "TestGame";

    let conveyor_asset = "/Game/Buildings/Conveyor";
    assert_eq!(
        game_to_absolute(game_name, conveyor_asset).expect("Failed to convert path"),
        "TestGame/Content/Buildings/Conveyor.uasset"
    );

    let map_asset = "/Game/Maps/Exotic.umap";
    assert_eq!(
        game_to_absolute(game_name, map_asset).expect("Failed to convert path"),
        "TestGame/Content/Maps/Exotic.umap"
    );

    let multiple_names = "/Game/Buildings/Game/Conveyor";
    assert_eq!(
        game_to_absolute(game_name, multiple_names).expect("Failed to convert path"),
        "TestGame/Content/Buildings/Game/Conveyor.uasset"
    );

    let no_game_name = "/Content/Vehicle";
    assert_eq!(game_to_absolute(game_name, no_game_name), None);
}
