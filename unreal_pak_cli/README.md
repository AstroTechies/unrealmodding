# unreal_pak_cli

[![Documentation](https://docs.rs/unreal_pak_cli/badge.svg)](https://docs.rs/unreal_pak_cli/)
[![Crates.io](https://img.shields.io/crates/v/unreal_pak_cli.svg)](https://crates.io/crates/unreal_pak_cli)
[![Build status](https://github.com/AstroTechies/unrealmodding/workflows/CI/badge.svg)](https://github.com/AstroTechies/unrealmodding/actions?query=workflow%3ACI)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

CLI for working with Unreal Engine `.pak` files.

## Installation

Install using cargo

```sh
cargo install unreal_pak_cli
```

## Usage

```text
Usage: unreal_pak_cli.exe <COMMAND>

Commands:
  check         Check an entire .pak file if it is valid
  check-header  Only check the header of a .pak file if it is valid
  extract       Extract a .pak file to a directory
  create        create a new .pak file from the files from a directory, optionally disabling compression
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Also available under

```sh
unreal_pak_cli -h
```

## Compatibility

See the [Compatibility of unreal_pak](../unreal_pak#Compatibility) for what `.pak` versions and features are supported.
