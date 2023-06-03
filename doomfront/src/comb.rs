//! Various combinators which are broadly applicable elsewhere.

use chumsky::{primitive, Parser};
use rowan::SyntaxKind;

use crate::{
	util::{builder::GreenCache, state::*},
	Extra, ParseError, TokenStream,
};

/// Shorthand for the following:
///
/// ```
/// primitive::just(token).map_with_state(move |_, sp, state: &mut ParseState<'i, C>| {
///     state.gtb.token(syn, &state.source[sp]);
/// })
/// ```
pub fn just<'i, T, C>(
	token: T,
	syn: SyntaxKind,
) -> impl Parser<'i, TokenStream<'i, T>, (), Extra<'i, T, C>> + Clone
where
	T: 'i + logos::Logos<'i, Error = ()> + PartialEq + Clone,
	C: GreenCache,
{
	primitive::just(token).map_with_state(move |_, span, state: &mut ParseState<'i, C>| {
		state.gtb.token(syn, &state.source[span]);
	})
}

/// Shorthand for the following:
///
/// ```
/// primitive::none_of(seq).map_with_state(move |_, sp, state: &mut ParseState<'i, C>| {
///     state.gtb.token(syn, &state.source[sp]);
/// })
/// ```
pub fn none_of<'i, T, S, C>(
	seq: S,
	syn: SyntaxKind,
) -> impl Parser<'i, TokenStream<'i, T>, (), Extra<'i, T, C>> + Clone
where
	T: 'i + logos::Logos<'i, Error = ()> + PartialEq + Clone,
	S: chumsky::container::Seq<'i, T> + Clone,
	C: GreenCache,
{
	primitive::none_of(seq).map_with_state(move |_, span, state: &mut ParseState<'i, C>| {
		state.gtb.token(syn, &state.source[span]);
	})
}

/// Matches `token` as long as it contains `text`, ASCII case-insensitively.
///
/// This is needed for (G)ZDoom DSLs, many of which are unspecified and use only an
/// ad-hoc parser as DoomFront's reference implementation. Representing every niche
/// keyword used by every one of these languages would add complexity to every parser
/// (since each would have to treat foreign keywords as identifiers), so instead
/// make the smaller languages look for their keywords through identifiers.
pub fn string_nc<'i, T, C>(
	token: T,
	text: &'static str,
	syn: SyntaxKind,
) -> impl Parser<'i, TokenStream<'i, T>, (), Extra<'i, T, C>> + Clone
where
	T: 'i + logos::Logos<'i, Error = ()> + PartialEq + Clone,
	C: GreenCache,
{
	primitive::just(token).try_map_with_state(
		move |_, sp: logos::Span, state: &mut ParseState<'i, C>| {
			let substr: &str = &state.source[sp.clone()];

			if substr.eq_ignore_ascii_case(text) {
				state.gtb.token(syn, substr);
				Ok(())
			} else {
				Err(ParseError::custom(
					sp,
					format!("expected `{}`, found `{}`", text, substr),
				))
			}
		},
	)
}

/// Shorthand for the following idiom:
///
/// ```
/// primitive::empty()
///     .map_with_state(/* open a new node */)
///     .then(primitive::group((
///         parser1,
///         parser2,
///         ...
///     )))
///     .map_with_state(/* close the current node */)
///     .map_err_with_state(/* cancel the node */)
/// ```
pub fn node<'i, T, O, C, G>(
	kind: SyntaxKind,
	group: G,
) -> impl Parser<'i, TokenStream<'i, T>, (), Extra<'i, T, C>> + Clone
where
	T: 'i + logos::Logos<'i, Error = ()> + PartialEq + Clone,
	C: GreenCache,
	G: Parser<'i, TokenStream<'i, T>, O, Extra<'i, T, C>> + Clone + 'i,
	O: 'i,
{
	primitive::empty()
		.map_with_state(move |_, _, state: &mut ParseState<'i, C>| {
			state.gtb.open(kind);
		})
		.then(group)
		.map_with_state(|_, _, state| {
			state.gtb.close();
		})
		.map_err_with_state(move |err, _, state| {
			state.gtb.cancel(kind);
			err
		})
		.boxed()
}

pub fn node_try_catch<'i, T, O, C, S, G>(
	kind: SyntaxKind,
	err_kind: SyntaxKind,
	start: S,
	group: G,
) -> impl Parser<'i, TokenStream<'i, T>, (), Extra<'i, T, C>> + Clone
where
	T: 'i + logos::Logos<'i, Error = ()> + PartialEq + Clone,
	C: GreenCache,
	S: Parser<'i, TokenStream<'i, T>, (), Extra<'i, T, C>> + Clone + 'i,
	G: Parser<'i, TokenStream<'i, T>, O, Extra<'i, T, C>> + Clone + 'i,
	O: 'i,
{
	primitive::group((
		primitive::empty().map_with_state(|_, _, state: &mut ParseState<'i, C>| {
			state.checkpoints.push(state.gtb.checkpoint());
		}),
		start,
		group.map_err_with_state(move |err, _, state| {
			state
				.gtb
				.open_at(state.checkpoints.pop().unwrap(), err_kind);
			state.gtb.close();
			err
		}),
	))
	.map_with_state(move |_, _, state| {
		state.gtb.open_at(state.checkpoints.pop().unwrap(), kind);
		state.gtb.close();
	})
	.boxed()
}

/// Shorthand for the following idiom:
///
/// ```
/// primitive::empty()
///     .map_with_state(/* withdraw a checkpoint from the green builder */)
///     .then(primitive::group((
///         parser1,
///         parser2,
///         ...
///     )))
///     .map(|_| ())
///     .map_err_with_state(/* cancel the most recent green builder checkpoint */)
/// ```
pub fn checkpointed<'i, T, O, C, G>(
	group: G,
) -> impl Parser<'i, TokenStream<'i, T>, (), Extra<'i, T, C>> + Clone
where
	T: 'i + logos::Logos<'i, Error = ()> + PartialEq + Clone,
	C: GreenCache,
	G: Parser<'i, TokenStream<'i, T>, O, Extra<'i, T, C>> + Clone + 'i,
	O: 'i,
{
	primitive::empty()
		.map_with_state(|_, _, state: &mut ParseState<'i, C>| {
			state.checkpoints.push(state.gtb.checkpoint());
		})
		.then(group)
		.map(|_| ())
		.map_err_with_state(|err, _, state| {
			state
				.gtb
				.cancel_checkpoint(state.checkpoints.pop().unwrap());

			err
		})
		.boxed()
}
