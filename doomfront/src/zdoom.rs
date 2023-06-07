//! Frontends for languages defined by the [ZDoom] family of source ports.
//!
//! [ZDoom]: https://zdoom.org/index

pub mod lex;

pub mod cvarinfo;
pub mod decorate;
pub mod language;
pub mod mapinfo;
pub mod zscript;

pub type Extra<'i, C> = crate::Extra<'i, lex::Token, C>;
pub type ParseTree<'i> = crate::ParseTree<'i, lex::Token>;
