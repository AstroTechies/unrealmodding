# unreal_helpers

The unreal_helpers crate is a Rust library that implements common functionality for reading Unreal Engine files.

## Documentation

Crate documentation is published to
[docs.rs/unreal_helpers](https://docs.rs/unreal_helpers/).

## Usage

The crate can be added to a Rust project as a dependency by running the command
`cargo add unreal_helpers`.

## Features

* `read_write` - enables read/write extensions for unreal binary files
* `bitvec` - enables bitvec extensions commonly used when working with unreal bitvecs
* `path` - enables path conversion extensions used to convert paths in pak files

## Examples

The [tests directory](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_helpers/tests) contains
several tests that demonstrate how to use the crate.
