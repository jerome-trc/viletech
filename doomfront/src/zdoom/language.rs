//! [LANGUAGE] is a file format for defining localized strings.
//!
//! [LANGUAGE]: https://zdoom.org/wiki/LANGUAGE

pub mod ast;
pub mod parse;
mod syn;

pub use syn::Syn;

pub type ParseTree = crate::ParseTree<Syn>;
pub type SyntaxNode = rowan::SyntaxNode<Syn>;
pub type SyntaxToken = rowan::SyntaxToken<Syn>;
pub type SyntaxElem = rowan::SyntaxElement<Syn>;
