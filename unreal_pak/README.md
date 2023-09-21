# unreal_pak

[![Documentation](https://docs.rs/unreal_pak/badge.svg)](https://docs.rs/unreal_pak/)
[![Crates.io](https://img.shields.io/crates/v/unreal_pak.svg)](https://crates.io/crates/unreal_pak)
[![Build status](https://github.com/AstroTechies/unrealmodding/workflows/CI/badge.svg)](https://github.com/AstroTechies/unrealmodding/actions?query=workflow%3ACI)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)

Library crate for working with Unreal Engine .pak files.

## Features

This crates support reading and writing most Unreal Engine `.pak` files.

There are multiple APIs available to fit your use case.

- [`PakReader`](https://docs.rs/unreal_pak/pakreader/struct.PakReader.html) for lazily reading large (multiple GB)
  `.pak` files. This reader only parses the relatively small index when loading a file and single entries can then
  be extarcted or all entries can be lazily read via in iterator based API.
- [`PakWriter`](https://docs.rs/unreal_pak/pakwriter/struct.PakWriter.html) for incrementally writing large `.pak`
  files.
- [`PakMemory`](https://docs.rs/unreal_pak/pakmemorey/struct.PakMemory.html) which is an entirely in-memory
  representation of a `.pak` file which allows arbitrary entries to be modified/added/removed. A file on disk can
  be loaded as a `PakMemory` or an empty one can be created. Once finsihed it can be writtin to disk all at once.

## Documentation

Crate documentation is published to [docs.rs/unreal_pak](https://docs.rs/unreal_pak/).

## Usage

The crate can be added to a Rust project as a dependency by running the command `cargo add unreal_pak`.

## Compatibility

| UE Version | Version | Version Feature       | Read               | Write              |
|------------|---------|-----------------------|--------------------|--------------------|
|            | 1       | Initial               | :grey_question:    | :grey_question:    |
| 4.0-4.2    | 2       | NoTimestamps          | :heavy_check_mark: | :heavy_check_mark: |
| 4.3-4.15   | 3       | CompressionEncryption | :heavy_check_mark: | :heavy_check_mark: |
| 4.16-4.19  | 4       | IndexEncryption       | :heavy_check_mark: | :heavy_check_mark: |
| 4.20       | 5       | RelativeChunkOffsets  | :heavy_check_mark: | :heavy_check_mark: |
|            | 6       | DeleteRecords         | :grey_question:    | :grey_question:    |
| 4.21       | 7       | EncryptionKeyGuid     | :heavy_check_mark: | :heavy_check_mark: |
| 4.22       | 8A      | FNameBasedCompression | :x:                | :x:                |
| 4.23-4.24  | 8B      | FNameBasedCompression | :heavy_check_mark: | :heavy_check_mark: |
| 4.25       | 9       | FrozenIndex           | :heavy_check_mark: | :heavy_check_mark: |
|            | 10      | PathHashIndex         | :grey_question:    | :grey_question:    |
| 4.26-4.27  | 11      | Fnv64BugFix           | :heavy_check_mark: | :x:                |

| Feature            | Read               | Write              |
|--------------------|--------------------|--------------------|
| Compression (Zlib) | :heavy_check_mark: | :heavy_check_mark: |
| Encrypted Index    | :x:                | :x:                |
| Encrypted Data     | :x:                | :x:                |

### Missing feature for your use case?

This crate was originally developed for use within [unrealmodding](https://github.com/AstroTechies/unrealmodding) and
therefore does not support every feature of `.pak` files. If you are looking to use this crate for your project but
need a currently unsupported feature please open an Issue/PR and we will try to help you.

## Examples

An example use of the crate can be found in the [unreal_pak_cli](../unreal_pak_cli) tool source.
