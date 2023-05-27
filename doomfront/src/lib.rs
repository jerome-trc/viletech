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
//! [`chumsky`] is used to build parsers up from combinators.
//! [`rowan`], used by [rust-analyzer], provides the basis for syntax representation.
//! It is recommended that you read its [overview] to understand the conceptual
//! foundation for the structures emitted by `doomfront`.
//!
//! `doomfront` is explicitly designed to be easy to extend.
//! Both of the aforementioned crates get re-exported in service of this.
//!
//! [rust-analyzer]: https://rust-analyzer.github.io/
//! [overview]: https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/syntax.md

pub extern crate chumsky;
pub extern crate rowan;

pub mod comb;
pub mod util;

#[cfg(feature = "zdoom")]
pub mod zdoom;

pub type ParseError<'i> = chumsky::error::Rich<'i, char>;
/// Defines the context and state passed along parsers as well as the error type they emit.
pub type Extra<'i, C> = chumsky::extra::Full<ParseError<'i>, util::state::ParseState<C>, ()>;

#[derive(Debug)]
pub struct ParseTree<'i> {
	pub root: rowan::GreenNode,
	pub errors: Vec<ParseError<'i>>,
}

impl ParseTree<'_> {
	/// Emits a "zipper" tree root that can be used for much more effective traversal
	/// of the green tree (but which is `!Send` and `!Sync`).
	#[must_use]
	pub fn cursor<L: rowan::Language>(&self) -> rowan::SyntaxNode<L> {
		rowan::SyntaxNode::new_root(self.root.clone())
	}
}

/// Each language has a `parse` module filled with functions that emit a combinator-
/// based parser. Pass one of these along with a source string into this function
/// to consume that parser and emit a green tree.
#[must_use]
pub fn parse<'i, C: 'i + util::builder::GreenCache>(
	parser: impl chumsky::Parser<'i, &'i str, (), Extra<'i, C>>,
	cache: Option<C>,
	root: rowan::SyntaxKind,
	source: &'i str,
) -> ParseTree<'i> {
	let mut state = util::state::ParseState::new(cache);

	state.gtb.open(root);

	let errors = parser.parse_with_state(source, &mut state).into_errors();

	state.gtb.close();

	ParseTree {
		root: state.gtb.finish(),
		errors,
	}
}

/// The most basic implementors of [`rowan::ast::AstNode`] are newtypes
/// (single-element tuple structs) which map to a single syntax tag. Automatically
/// generating `AstNode` implementations for these is trivial.
#[macro_export]
macro_rules! simple_astnode {
	($lang:ty, $node:ty, $syn_kind:expr) => {
		impl AstNode for $node {
			type Language = $lang;

			fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
			where
				Self: Sized,
			{
				kind == $syn_kind
			}

			fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
			where
				Self: Sized,
			{
				if node.kind() == $syn_kind {
					Some(Self(node))
				} else {
					None
				}
			}

			fn syntax(&self) -> &rowan::SyntaxNode<Self::Language> {
				&self.0
			}
		}
	};
}
