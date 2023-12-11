//! Frontend for the [ZScript](https://zdoom.org/wiki/ZScript)
//! language defined by GZDoom.
//!
//! ZScript is GZDoom's primary and most well-supported embedded scripting
//! language, intended as a successor to (and superset of) ZDoom's DECORATE.

pub mod ast;
pub mod autofmt;
pub mod parse;
mod syntax;

pub use syntax::Syntax;

pub type IncludeTree = super::inctree::IncludeTree<Syntax>;
pub type ParseTree = crate::ParseTree<Syntax>;
pub type SyntaxNode = rowan::SyntaxNode<Syntax>;
pub type SyntaxToken = rowan::SyntaxToken<Syntax>;
pub type SyntaxElem = rowan::SyntaxElement<Syntax>;
