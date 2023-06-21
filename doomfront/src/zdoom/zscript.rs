//! Frontend for the [ZScript](https://zdoom.org/wiki/ZScript)
//! language defined by GZDoom.
//!
//! ZScript is GZDoom's primary and most well-supported embedded scripting
//! language, intended as a successor to (and superset of) ZDoom's DECORATE.

pub mod ast;
pub mod parse;
mod syn;

pub use syn::Syn;

pub type IncludeTree = super::inctree::IncludeTree<Syn>;
pub type ParseTree = crate::ParseTree<Syn>;
pub type SyntaxNode = rowan::SyntaxNode<Syn>;
pub type SyntaxToken = rowan::SyntaxToken<Syn>;
pub type SyntaxElem = rowan::SyntaxElement<Syn>;
