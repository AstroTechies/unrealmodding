# unreal_modintegrator

Library crate for "integrating" Unreal Engine game mods.

## Build setup

This crate needs some special envoirement variables to be set becasue it run Unreal Engine at compile time to cook assets for the correct Unreal Engine version.

- `UE_PATH`: Path to your Unreal Engine installation. Typically `C:\Program FIles\Epic Games\UE_4.xx\`.
- `UE_VERSION_SELECTOR`: Path to Unreal Version Selector. Typically `C:\Program Files (x86)\Epic Games\Launcher\Engine\Binaries\Win64\UnrealVersionSelector.exe`.
