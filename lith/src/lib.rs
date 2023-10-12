//! # Lithica

#![doc(
	html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png",
	html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech/viletech.png"
)]

pub(crate) mod compile;
pub(crate) mod data;
pub(crate) mod intern;

pub mod arena;
pub mod ast;
pub mod filetree;
pub mod issue;
pub mod parse;
pub mod rti;
pub mod runtime;
pub mod syn;

pub use self::{compile::Compiler, syn::Syn};

pub type ParseTree = doomfront::ParseTree<Syn>;
pub type SyntaxElem = doomfront::rowan::SyntaxElement<Syn>;
pub type SyntaxNode = doomfront::rowan::SyntaxNode<Syn>;
pub type SyntaxToken = doomfront::rowan::SyntaxToken<Syn>;

pub(crate) type FxDashMap<K, V> =
	dashmap::DashMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
pub(crate) type FxDashView<K, V> =
	dashmap::ReadOnlyView<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
