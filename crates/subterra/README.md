# VileData

## About

VileData is a library providing data structures for representing (and procedures for reading, writing, introspecting, and manipulating) formats that are relevant to anyone building id Tech 1-descendant technology, such as a Doom source port.

## Feature Flags

`acs` - Enables support for reading the compiled bytecode object files of Raven Software's [Action Code Script](https://doomwiki.org/wiki/ACS).

`serde` - Enables `Serialize`/`Deserialize` implementations for VileData's structures to allow usage with the [Serde](https://serde.rs/) crate.
