# unreal_pak

Library crate for working with Unreal Engine .pak files.

Features:

- Supports reading and writing .pak files.
- Can partially read files or load them entirely into memory depending on you API needs.
- Aims to support all .pak versions. Currently all can be read but only writing up to version 9.
- Supports compression with Zlib.
- Encryption support planned.
