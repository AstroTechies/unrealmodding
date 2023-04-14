# unreal_asset

The unreal_asset crate is a Rust library that allows parsing of Unreal Engine binary asset files.

## Documentation

Crate documentation is published to
[docs.rs/unreal_asset](https://docs.rs/unreal_asset/).

## Unreal Engine binary asset files

Unreal Engine binary asset files are usually stored with the following extensions:
* `.uasset` - The file that has asset metadata as well as some information, if the game is built without split bulk data files, then it also contains all of asset data.
* `.uexp` - If the game is built with split bulk data files, it contains binary data related to components, etc.
* `.umap` - Same as `.uasset` but for maps/levels.
* `.usmap` - Mapping files for reading unversioned assets.

These files are what stores most of the game's assets and what you might want to modify to mod a specific game.

## Usage

The crate can be added to a Rust project as a dependency by running the command
`cargo add unreal_asset`.

## Features

* `oodle` - allows reading Oodle compressed asset files

## Examples

The example code provided below demonstrates how to use the unreal_asset crate to read
an asset file that was cooked for a game with split bulk files enabled.

```rust
use unreal_asset::{engine_version::EngineVersion, Asset};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
};

let mut data_file = File::open("NPC_Onop_IO_Bech.uasset")?;
let mut bulk_file = File::open("NPC_Onop_IO_Bech.uexp")?;

let asset = Asset::new(data_file, Some(bulk_file), EngineVersion::VER_UE4_25)?;
println!("{:#?}", asset);
```

The [tests directory](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_asset/tests) contains
several tests that demonstrate how to use the crate to work with uasset files.

## License

This library is distributed under the terms of the MIT license. See the
[LICENSE](LICENSE) file for details.
