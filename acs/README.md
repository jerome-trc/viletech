# VileTech ACS

The crate within represents the part of VileTech responsible for reading and consuming Raven Software's [Action Code Script](https://doomwiki.org/wiki/ACS).

At minimum, this contains a parser for its bytecode object files to be used by a virtual machine; it may eventually grow to include a [Cranelift](https://cranelift.dev/) backend for JIT compilation, and a whole ACS toolchain at most.

## Feature Flags

`parallel` - enables certain demanding routines to take advantage of the global [Rayon](https://crates.io/crates/rayon) thread pool.
