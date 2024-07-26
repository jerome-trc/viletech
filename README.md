# VileTech

## About

VileTech is a collection of Doom-related Rust technologies, oriented towards the building of new tools, game engines, and games descending from id Software's id Tech 1 engine.

The goals of the VileTech "project" are as follows, in descending order of priority:
1. [![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)
2. Build a new Doom source port that fulfills all my specific needs.
3. Publicly expose the technologies developed in service of 2., especially where some functionality did not already have an implementation in an available library.

Beware that this project:
- is deep in development. You should not assume that any of the code herein is even sound.
- is strictly a solo hobby project. The code within this repository is going to be deeply disorganized for the foreseeable future.

## Contents

- `/client` contains the code for a Zig static library.
- `/crates` contains Rust libraries associated with this project.
- `/engine` is the [dsda-doom](https://github.com/kraflab/dsda-doom) source which consumes the static library built in `/client`.

## Licensing, Attribution

A complete list of attributions and other third-party licensing information can be found [here](/ATTRIB.md).

All VileTech-original source - i.e., that which is no way covered by the terms of the document provided above - is provided under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
