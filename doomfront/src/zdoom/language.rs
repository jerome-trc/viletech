//! [LANGUAGE](https://zdoom.org/wiki/LANGUAGE) is a language for defining
//! localized strings.

pub mod ast;
pub mod parse;
mod syn;

pub use syn::Syn;

pub type ParseTree<'i> = crate::ParseTree<'i, crate::zdoom::Token, Syn>;
pub type SyntaxNode = rowan::SyntaxNode<Syn>;
pub type SyntaxToken = rowan::SyntaxToken<Syn>;
pub type SyntaxElem = rowan::SyntaxElement<Syn>;
