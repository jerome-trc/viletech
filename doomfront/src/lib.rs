//! # `doomfront`
//!
//! ## About
//!
//! Comprehensive suite of frontends for domain-specific languages written for
//! Doom's source ports.
//!
//! Within this documentation, the term "lump" is used as a catch-all term for
//! a filesystem entry of some kind, whether that be a real file, a WAD archive
//! entry, or some other compressed archive entry.
//!
//! [`logos`] is used to procedurally generate lexers.
//! [`rowan`], used by [rust-analyzer], provides the basis for syntax representation.
//! It is recommended that you read its [overview] to understand the conceptual
//! foundation for the structures emitted by `doomfront`.
//!
//! `doomfront` is explicitly designed to be easy to extend.
//! All of the aforementioned crates get re-exported in service of this.
//!
//! [rust-analyzer]: https://rust-analyzer.github.io/
//! [overview]: https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/syntax.md

pub extern crate logos;
pub extern crate rowan;

pub mod gcache;
pub mod parser;
pub mod testing;

#[cfg(feature = "zdoom")]
pub mod zdoom;

/// Ties a [`rowan::Language`] to a [`logos::Logos`] token.
///
/// To developers looking to use DoomFront on their own language, note that `Token`'s
/// error type is also `Token`. This allows Logos to emit either an "unknown"
/// token type (which should correspond to the return value of `T::default()`),
/// or leverage its `Extras` type to produce different output context-sensitively
/// (e.g. adding more keywords with newer language versions).
pub trait LangExt: rowan::Language {
	type Token: 'static
		+ for<'i> logos::Logos<'i, Source = str, Error = Self::Token>
		+ Eq
		+ Copy
		+ Default;

	const EOF: Self::Token;
	const ERR_NODE: Self::Kind;
}

pub type GreenElement = rowan::NodeOrToken<rowan::GreenNode, rowan::GreenToken>;
pub type ParseError<L> = parser::Error<L>;

pub struct ParseTree<L: LangExt> {
	pub root: rowan::GreenNode,
	pub errors: Vec<ParseError<L>>,
}

impl<L: LangExt> ParseTree<L> {
	#[must_use]
	pub fn new(root: rowan::GreenNode, errors: Vec<ParseError<L>>) -> Self {
		Self { root, errors }
	}

	/// Emits a "zipper" tree root that can be used for much more effective traversal
	/// of the green tree (but which is `!Send` and `!Sync`).
	#[must_use]
	pub fn cursor(&self) -> rowan::SyntaxNode<L> {
		rowan::SyntaxNode::new_root(self.root.clone())
	}
}

impl<L: LangExt> std::fmt::Debug for ParseTree<L>
where
	L::Token: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ParseTree")
			.field("root", &self.root)
			.field("errors", &self.errors)
			.finish()
	}
}

#[must_use]
pub fn parse<'i, L: LangExt>(
	source: &'i str,
	function: fn(&mut parser::Parser<L>),
	lexer_ctx: <<L as LangExt>::Token as logos::Logos<'i>>::Extras,
) -> ParseTree<L> {
	let mut parser = parser::Parser::new(source, lexer_ctx);
	function(&mut parser);
	let (root, errors) = parser.finish();
	ParseTree::new(root, errors)
}

/// The most basic implementors of [`rowan::ast::AstNode`] are newtypes
/// (single-element tuple structs) which map to a single syntax tag. Automatically
/// generating `AstNode` implementations for these is trivial.
#[macro_export]
macro_rules! simple_astnode {
	($lang:ty, $node:ty, $syn_kind:expr) => {
		impl $crate::rowan::ast::AstNode for $node {
			type Language = $lang;

			fn can_cast(kind: <Self::Language as $crate::rowan::Language>::Kind) -> bool
			where
				Self: Sized,
			{
				kind == $syn_kind
			}

			fn cast(node: $crate::rowan::SyntaxNode<Self::Language>) -> Option<Self>
			where
				Self: Sized,
			{
				if node.kind() == $syn_kind {
					Some(Self(node))
				} else {
					None
				}
			}

			fn syntax(&self) -> &$crate::rowan::SyntaxNode<Self::Language> {
				&self.0
			}
		}
	};
}
