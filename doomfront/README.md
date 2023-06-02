# DoomFront

## About

DoomFront aims to be a comprehensive suite of language frontends for the myriad of domain-specific languages recognised by the collective ecosystem of Doom source ports, including those of the [ZDoom](https://zdoom.org/index) family, [Eternity Engine](https://eternity.youfailit.net/wiki/Main_Page), ACS, DeHackEd, and UMAPINFO.

DoomFront uses the [Rowan](https://crates.io/crates/rowan) crate (see the attributions section) - which itself serves as the foundation for [rust-analyzer](https://rust-analyzer.github.io/) - to generate lossless syntax trees that are completely representative of the parsed source and easy to traverse.

## Feature Flags

- `acs` - Enables frontends for Raven Software's [Action Code Script](https://doomwiki.org/wiki/ACS) (ACS) used in Hexen, Heretic, and Strife.
- `eternity` - Enables frontends for DSLs used by the [Eternity Engine](https://eternity.youfailit.net/wiki/Main_Page).
- `umapinfo` - Enables a frontend for the [UMAPINFO](https://doomwiki.org/wiki/UMAPINFO) text file format.
- `zdoom` - Enables frontends for DSLs used by [ZDoom-family ports](https://zdoom.org).

- `parallel` - Enables extra symbols for enabling multi-threaded parsing.
- `ser_de` - Enables serialization and deserialization of emitted structures via the [serde](https://serde.rs/) crate.
