//! # Lithica

pub(crate) mod intern;

pub mod ast;
pub mod lex;
pub mod parse;

pub use lex::Syn;

pub type FxDashMap<K, V> =
	dashmap::DashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

pub type ParseTree = doomfront::ParseTree<Syn>;
pub type SyntaxElem = doomfront::rowan::SyntaxElement<Syn>;
pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syn>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syn>;
