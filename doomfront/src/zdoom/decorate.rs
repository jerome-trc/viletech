//! Frontend for the [`DECORATE`](https://zdoom.org/wiki/DECORATE)
//! language defined by (G)ZDoom.
//!
//! DECORATE is a data definition language and pseudo-scripting language for
//! creating new game content.

pub mod ast;
pub mod parse;
mod syn;

#[cfg(test)]
mod test;

pub use syn::Syn;

pub type SyntaxNode = rowan::SyntaxNode<Syn>;
pub type SyntaxToken = rowan::SyntaxToken<Syn>;
pub type SyntaxElem = rowan::SyntaxElement<Syn>;
