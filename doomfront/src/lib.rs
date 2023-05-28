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
//! [`chumsky`] is used to build parsers up from combinators.
//! [`rowan`], used by [rust-analyzer], provides the basis for syntax representation.
//! It is recommended that you read its [overview] to understand the conceptual
//! foundation for the structures emitted by `doomfront`.
//!
//! `doomfront` is explicitly designed to be easy to extend.
//! All of the aforementioned crates get re-exported in service of this.
//!
//! [rust-analyzer]: https://rust-analyzer.github.io/
//! [overview]: https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/syntax.md

pub extern crate chumsky;
pub extern crate logos;
pub extern crate rowan;

pub mod comb;
pub mod util;

#[cfg(feature = "zdoom")]
pub mod zdoom;

pub type TokenMapper<T> = fn((Result<T, ()>, logos::Span)) -> (T, logos::Span);
pub type Lexer<'i, T> = std::iter::Map<logos::SpannedIter<'i, T>, TokenMapper<T>>;
pub type TokenStream<'i, T> =
	chumsky::input::SpannedInput<T, logos::Span, chumsky::input::Stream<Lexer<'i, T>>>;

pub type ParseError<'i, T> = chumsky::error::Rich<'i, T, logos::Span>;
/// Defines the context and state passed along parsers as well as the error type they emit.
pub type Extra<'i, T, C> =
	chumsky::extra::Full<ParseError<'i, T>, util::state::ParseState<'i, C>, ()>;

#[derive(Debug)]
pub struct ParseTree<'i, T: logos::Logos<'i>> {
	pub root: rowan::GreenNode,
	pub errors: Vec<ParseError<'i, T>>,
}

impl<'i, T: logos::Logos<'i>> ParseTree<'i, T> {
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
pub fn parse<'i, P, T, C>(
	parser: P,
	cache: Option<C>,
	root: rowan::SyntaxKind,
	source: &'i str,
	stream: TokenStream<'i, T>,
) -> ParseTree<'i, T>
where
	P: chumsky::Parser<'i, TokenStream<'i, T>, (), Extra<'i, T, C>>,
	T: logos::Logos<'i, Source = str, Error = ()> + PartialEq + Clone,
	C: 'i + util::builder::GreenCache,
{
	let mut state = util::state::ParseState::new(source, cache);

	state.gtb.open(root);

	let errors = parser.parse_with_state(stream, &mut state).into_errors();

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
