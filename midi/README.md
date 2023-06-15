# Sunrider

Sunrider is the crate representing VileTech's code for reading, writing, and playing the [MIDI](https://doomwiki.org/wiki/MIDI) and [DMXMUS](https://doomwiki.org/wiki/MUS) file formats.

The final intended scope of this library is yet to be determined.

## Feature Flags

- `parallel` - enables the `parallel` feature flag on the [`midly`](https://crates.io/crates/midly) dependency, when allows parallelized parsing of very large MIDI files using the [Rayon](https://crates.io/crates/rayon) crate.
