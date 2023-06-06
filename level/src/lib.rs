//! # Subterra
//!
//! Subterra is the code used by the VileTech Engine for reading, storing,
//! manipulating, and writing Doom levels, whether vanilla or UDMF.

pub mod repr;
pub mod udmf;

pub use repr::Level;
