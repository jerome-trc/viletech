# VileTech

## About

VileTech is a collection of Doom-related Rust technologies, oriented towards the building of new tools, game engines, and games descending from id Software's id Tech 1 engine.

The goals of the VileTech "project" are as follows, in descending order of priority:
1. Facilitate the development of a catch-all [language server](https://github.com/jerome-trc/doom-ls) for Doom content development.
2. Expose functionality developed in the Doom open-source ecosystem which is currently unavailable (i.e. due to being tied up in existing applications) as a public API, first with a Rust interface and then with a C API.
3. Build a new Doom source port as an alternative to [GZDoom](https://zdoom.org), with the orthogonal goals of:
	- Implementing improvements which are impractical for GZDoom due to technical debt.
	- Adding features which fall outside GZDoom's scope.
	- Eliminating compatibility for the 1% of legacy user content which GZDoom supports to its detriment.

## Libraries

### cli

viletech-cli is a command-line interface for a set of tools built on the other parts of VileTech for performing common operations related to Doom modification, like building binary-space partition data for levels or compiling Action Code Script modules.

### client

viletech-client runs games, is the primary way for end-users to interact with the engine.

### doomfront

[doomfront](/doomfront/README.md) is a collection of frontends for Doom-related domain-specific languages.

### engine

viletech is a crate for backing the client and the dedicated server by rolling up all the other constituent parts of this repository and adding features needed to run games, like physics simulation.

### server

viletech-server is a dedicated (a.k.a. "headless") game simulation runner with a CLI for serving other clients.

### utils

viletech-utils is where small helper symbols which may not necessarily be related to Doom but are still useful to multiple other crates go.

### vfs

viletech-fs is a "virtual file system"; an abstraction over the user's "real" or "physical" operating system's FS to ease usage for the engine and mod developers, as well as providing a layer of "information hiding".

### vzscript

[vzscript](/vzscript/README.md), the VZSC toolchain, is the infrastructure powering VZScript, VileTech's bespoke embedded scripting language.

### wadload

wadload contains functionality for reading and writing files in the "Where's All the Data" or WAD file format native to Doom.

## Licensing, Attribution

A complete list of attributions and other third-party licensing information can be found [here](/ATTRIB.md).

All VileTech-original source - i.e., that which is no way covered by the terms of the document provided above - is provided under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
