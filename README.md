# VileTech

## About

VileTech is a collection of Doom-related Rust technologies, oriented towards the building of new tools, game engines, and games descending from id Software's id Tech 1 engine.

The goals of the VileTech "project" are as follows, in descending order of priority:
1. [![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)
2. Build a Doom source port that fulfills my specific needs.
3. Facilitate the development of a catch-all [language server](https://github.com/jerome-trc/doom-ls) for Doom content development.
4. Expose functionality developed in the Doom open-source ecosystem which is currently unavailable (i.e. due to being tied up in existing applications) as a public API, first with a Rust interface and then with a C API.

Beware that this project:
- is deep in development. None of these crates are published for good reason; nothing is feature complete or rigorously-tested. You should not assume that an interface which isn't marked `unsafe` is safe.
- is strictly a solo hobby project. The code within this repository is going to be deeply disorganized for the foreseeable future.

## Licensing, Attribution

A complete list of attributions and other third-party licensing information can be found [here](/ATTRIB.md).

All VileTech-original source - i.e., that which is no way covered by the terms of the document provided above - is provided under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
