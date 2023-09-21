# VileTech ACS

The crate within represents the part of VileTech responsible for reading and consuming Raven Software's [Action Code Script](https://doomwiki.org/wiki/ACS). In particular, it is a reader for ACS' binary object files and structures for representing their content.

## Feature Flags

`parallel` - enables certain demanding routines to take advantage of the global [Rayon](https://crates.io/crates/rayon) thread pool.
