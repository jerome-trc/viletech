# VileTech Utils

An assortment of small helper symbols used by multiple other VileTech crates.

This also holds "VTBench", a catch-all harness for microbenchmarks that do not belong to any other library in this repository. This is to enable testing the performance of VileTech-relevant functions, patterns, and techniques not just in isolation but also over time, between compiler versions/configurations, and across different platforms.

## Feature Flags

`serde` - Enables `Serialize`/`Deserialize` implementations for provided structures to allow usage with the [Serde](https://serde.rs/) crate.
