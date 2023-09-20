# unreal_helpers

[![Documentation](https://docs.rs/unreal_helpers/badge.svg)](https://docs.rs/unreal_helpers/)
[![Crates.io](https://img.shields.io/crates/v/unreal_helpers.svg)](https://crates.io/crates/unreal_helpers)
[![Build status](https://github.com/AstroTechies/unrealmodding/workflows/CI/badge.svg)](https://github.com/AstroTechies/unrealmodding/actions?query=workflow%3ACI)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

The `unreal_helpers` crate is a Rust library that implements common functionality for reading Unreal Engine files.

## Documentation

Crate documentation is published to [docs.rs/unreal_helpers](https://docs.rs/unreal_helpers/).

## Usage

The crate can be added to a Rust project as a dependency by running the command `cargo add unreal_helpers`.

## Features

All content in this crate is hidden behind feature flags. Enabling most features will also enable further dependencies.

* `read_write`: Enables extension Traits `UnrealReadExt` and `UnrealWriteExt` which help with parsing Unreal data formats.
* `path`: Enables `game_to_absolute` function.
* `guid`: Enables `Guid` type.
* `serde`: Enables `serde` support for `Guid` type.
* `bitvec`: Enables extension Trait `BitVecExt`.

## Examples

The [tests directory](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_helpers/tests) contains
several tests that demonstrate how to use the crate.
