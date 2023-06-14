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
pub mod gcache;
pub mod parsing;
pub mod testing;

#[cfg(feature = "zdoom")]
pub mod zdoom;

pub type GreenElement = rowan::NodeOrToken<rowan::GreenNode, rowan::GreenToken>;
pub type ParseError<'i, T> = chumsky::error::Rich<'i, T, logos::Span>;
pub type ParseState<'i> = parsing::State<'i>;

#[derive(Debug)]
pub struct ParseTree<'i, T, L>
where
	for<'input> T: logos::Logos<'i, Source = str> + Copy,
	L: rowan::Language + Into<rowan::SyntaxKind>,
{
	pub root: rowan::GreenNode,
	pub errors: Vec<ParseError<'i, T>>,
	phantom: std::marker::PhantomData<L>,
}

impl<'i, T, L> ParseTree<'i, T, L>
where
	for<'input> T: logos::Logos<'input, Source = str> + Copy,
	L: rowan::Language + Into<rowan::SyntaxKind>,
{
	#[must_use]
	pub fn new(root: rowan::GreenNode, errors: Vec<ParseError<'i, T>>) -> Self {
		Self {
			root,
			errors,
			phantom: std::marker::PhantomData,
		}
	}

	/// Emits a "zipper" tree root that can be used for much more effective traversal
	/// of the green tree (but which is `!Send` and `!Sync`).
	#[must_use]
	pub fn cursor(&self) -> rowan::SyntaxNode<L> {
		rowan::SyntaxNode::new_root(self.root.clone())
	}
}

/// Produces the input needed for [`parse`].
///
/// To developers looking to use DoomFront on their own language, note that
/// `T`'s error type is also `T`. This allows Logos to emit either an "unknown"
/// token type (which should correspond to the return value of `T::default()`),
/// or leverage its `Extras` type to produce different output context-sensitively
/// (e.g. adding more keywords with newer language versions).
#[must_use]
pub fn scan<'i, T>(source: &'i str, extras: T::Extras) -> Vec<(T, logos::Span)>
where
	T: 'i + logos::Logos<'i, Source = str, Error = T> + Eq + Copy + Default,
{
	let lexer = T::lexer_with_extras(source, extras)
		.spanned()
		.map(|(result, span)| {
			(
				match result {
					Ok(t) | Err(t) => t,
				},
				span,
			)
		});

	lexer.collect::<Vec<_>>()
}

/// Each language has a `parse` module filled with functions that emit a combinator-
/// based parser. Pass one of these along with a source string into this function
/// to consume that parser and emit a green tree.
#[must_use]
pub fn parse<'i, T, L>(
	parser: parser_t!(T, rowan::GreenNode),
	source: &'i str,
	tokens: &'i [(T, logos::Span)],
) -> Result<ParseTree<'i, T, L>, Vec<ParseError<'i, T>>>
where
	T: 'i + logos::Logos<'i, Source = str, Error = T> + Eq + Copy + Default,
	L: rowan::Language + Into<rowan::SyntaxKind>,
{
	use chumsky::input::Input;

	let input = tokens.spanned(source.len()..source.len());

	let mut state = ParseState { source };

	let (output, errors) = parser
		.parse_with_state(input, &mut state)
		.into_output_errors();

	if let Some(root) = output {
		Ok(ParseTree {
			root,
			errors,
			phantom: std::marker::PhantomData,
		})
	} else {
		Err(errors)
	}
}

/// A macro for writing the signatures of functions which return [`chumsky`]
/// combinators in a more succinct and maintainable way.
///
/// The one-parameter overload assumes that the name `Token` (implementing
/// [`logos::Logos`]) is already in scope for convenience.
#[macro_export]
macro_rules! parser_t {
	($out_t:ty) => {
		impl 'i + $crate::chumsky::Parser<
			'i,
			$crate::chumsky::input::SpannedInput<
				Token,
				logos::Span,
				&'i [(Token, logos::Span)]
			>,
			$out_t,
			$crate::chumsky::extra::Full<
				$crate::chumsky::error::Rich<'i, Token, logos::Span>,
				$crate::ParseState<'i>,
				()
			>
		> + Clone
	};
	($token_t:ty, $out_t:ty) => {
		impl 'i + $crate::chumsky::Parser<
			'i,
			$crate::chumsky::input::SpannedInput<
				$token_t,
				logos::Span,
				&'i [($token_t, logos::Span)]
			>,
			$out_t,
			$crate::chumsky::extra::Full<
				$crate::chumsky::error::Rich<'i, $token_t, logos::Span>,
				$crate::ParseState<'i>,
				()
			>
		> + Clone
	};
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
