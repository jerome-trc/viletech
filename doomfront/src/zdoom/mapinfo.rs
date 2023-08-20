//! [MAPINFO] is a multi-purpose file format, intended primarily for configuring levels.
//!
//! [MAPINFO]: https://zdoom.org/wiki/MAPINFO

pub mod parse;
mod syn;

pub use syn::Syn;

pub type ParseTree = crate::ParseTree<Syn>;
pub type SyntaxNode = rowan::SyntaxNode<Syn>;
pub type SyntaxToken = rowan::SyntaxToken<Syn>;
pub type SyntaxElem = rowan::SyntaxElement<Syn>;
