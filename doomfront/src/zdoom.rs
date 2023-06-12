//! Frontends for languages defined by the [ZDoom] family of source ports.
//!
//! [ZDoom]: https://zdoom.org/index

pub mod lex;

pub mod language;
pub mod zscript;

pub use lex::Token;
