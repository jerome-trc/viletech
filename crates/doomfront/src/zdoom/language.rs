//! [LANGUAGE] is a file format for defining localized strings.
//!
//! [LANGUAGE]: https://zdoom.org/wiki/LANGUAGE

pub mod ast;
pub mod parse;
mod syntax;

pub use syntax::Syntax;

pub type ParseTree = crate::ParseTree<Syntax>;
pub type SyntaxNode = rowan::SyntaxNode<Syntax>;
pub type SyntaxToken = rowan::SyntaxToken<Syntax>;
pub type SyntaxElem = rowan::SyntaxElement<Syntax>;
