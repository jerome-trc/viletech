//! Frontend for the [`CVARINFO`](https://zdoom.org/wiki/CVARINFO)
//! language defined by ZDoom-family source ports.
//!
//! Console variables or "CVars" are (G)ZDoom's way of storing user preferences
//! and the de facto solution for persistent storage.

pub mod ast;
pub mod parse;
pub mod syn;

pub use syn::Syn;

pub type SyntaxNode = rowan::SyntaxNode<Syn>;
pub type SyntaxToken = rowan::SyntaxToken<Syn>;
pub type SyntaxElem = rowan::SyntaxElement<Syn>;
