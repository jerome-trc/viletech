//! Frontend for the [`CVARINFO`](https://zdoom.org/wiki/CVARINFO)
//! language defined by ZDoom-family source ports.
//!
//! Console variables or "CVars" are (G)ZDoom's way of storing user preferences
//! and the de facto solution for persistent storage.

pub mod ast;
pub mod parse;
mod syntax;

pub use syntax::Syntax;

pub type ParseTree = crate::ParseTree<Syntax>;
pub type SyntaxNode = rowan::SyntaxNode<Syntax>;
pub type SyntaxToken = rowan::SyntaxToken<Syntax>;
pub type SyntaxElem = rowan::SyntaxElement<Syntax>;
