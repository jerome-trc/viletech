//! [MAPINFO] is a multi-purpose file format, intended primarily for configuring levels.
//!
//! [MAPINFO]: https://zdoom.org/wiki/MAPINFO

pub mod parse;
mod syntax;

pub use syntax::Syntax;

pub type ParseTree = crate::ParseTree<Syntax>;
pub type SyntaxNode = rowan::SyntaxNode<Syntax>;
pub type SyntaxToken = rowan::SyntaxToken<Syntax>;
pub type SyntaxElem = rowan::SyntaxElement<Syntax>;
