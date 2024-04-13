//! Frontend for the [`DECORATE`](https://zdoom.org/wiki/DECORATE)
//! language defined by (G)ZDoom.
//!
//! DECORATE is a data definition language and pseudo-scripting language for
//! creating new game content.

pub mod ast;
pub mod parse;
mod syntax;

pub use syntax::Syntax;

pub type ParseTree = crate::ParseTree<Syntax>;
pub type IncludeTree = super::inctree::IncludeTree<Syntax>;
pub type SyntaxNode = rowan::SyntaxNode<Syntax>;
pub type SyntaxToken = rowan::SyntaxToken<Syntax>;
pub type SyntaxElem = rowan::SyntaxElement<Syntax>;
