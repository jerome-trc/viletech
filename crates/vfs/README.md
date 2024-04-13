# VileTechFS

VileTech's virtual file system; an abstraction over the operating system's "physical" FS. Real files, directories, and various archives are all merged into one tree so that reading from them is more convenient at all other levels of the engine, without exposing any details of the user's underlying machine.

## Feature Flags

`bevy` - Adds an implementation of the `bevy::ecs::system::Resource` trait to `VirtualFs`, allowing it to be used as-is with the [Bevy](https://bevyengine.org/) game engine.
`egui` - Adds a function to the `VirtualFs` structure that draws diagnostic information into an [`egui`](https://crates.io/crates/egui) container.
`serde` - Enables `Serialize`/`Deserialize` implementations for VFS state and related symbols to allow usage with the [Serde](https://serde.rs/) crate.
