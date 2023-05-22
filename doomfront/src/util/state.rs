use chumsky::span::SimpleSpan;
use rowan::SyntaxKind;

use super::builder::{Checkpoint, GreenBuilder, GreenCache};

#[derive(Debug, Default)]
pub struct ParseState<C: GreenCache> {
	pub gtb: GreenBuilder<C>,
	checkpoints: Vec<Checkpoint>,
}

impl<C: GreenCache> ParseState<C> {
	#[must_use]
	pub fn new(green_cache: Option<C>) -> Self {
		Self {
			gtb: GreenBuilder::new(green_cache),
			checkpoints: vec![],
		}
	}
}

/// See [`GreenBuilder::token`].
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_token<C: GreenCache>(
	kind: SyntaxKind,
) -> impl Clone + Fn(&str, SimpleSpan, &mut ParseState<C>) {
	move |text, _, state| {
		state.gtb.token(kind, text);
	}
}

/// Like [`gtb_token`], but for `Option<&str>`.
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_token_opt<C: GreenCache>(
	kind: SyntaxKind,
) -> impl Clone + Fn(Option<&str>, SimpleSpan, &mut ParseState<C>) {
	move |opt, _, state| {
		let Some(text) = opt else { return; };
		state.gtb.token(kind, text);
	}
}

/// See [`GreenBuilder::open`].
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_open<C: GreenCache, I>(
	node_kind: SyntaxKind,
) -> impl Clone + Fn(I, SimpleSpan, &mut ParseState<C>) {
	move |_, _, state| {
		state.gtb.open(node_kind);
	}
}

/// Combines [`gtb_open`] with a subsequent [`gtb_token`].
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_open_with<C: GreenCache>(
	node_kind: SyntaxKind,
	tok_kind: SyntaxKind,
) -> impl Clone + Fn(&str, SimpleSpan, &mut ParseState<C>) {
	move |text, _, state| {
		state.gtb.open(node_kind);
		state.gtb.token(tok_kind, text);
	}
}

/// Combines [`gtb_open`] with a subsequent [`gtb_token`] and then [`gtb_close`].
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_open_close<C: GreenCache>(
	node_kind: SyntaxKind,
	tok_kind: SyntaxKind,
) -> impl Clone + Fn(&str, SimpleSpan, &mut ParseState<C>) {
	move |text, _, state| {
		state.gtb.open(node_kind);
		state.gtb.token(tok_kind, text);
		state.gtb.close();
	}
}

/// See [`GreenBuilder::cancel`].
/// Pass the return value of this to [`chumsky::Parser::map_err_with_state`].
pub fn gtb_cancel<C: GreenCache, E>(
	kind: SyntaxKind,
) -> impl Clone + Fn(E, SimpleSpan, &mut ParseState<C>) -> E {
	move |err, _, state| {
		state.gtb.cancel(kind);
		err
	}
}

/// See [`GreenBuilder::cancel`].
/// Pass the return value of this to [`chumsky::Parser::map_err_with_state`].
pub fn gtb_cancel_if<C: GreenCache, E>(
	predicate: fn(SyntaxKind) -> bool,
) -> impl Clone + Fn(E, SimpleSpan, &mut ParseState<C>) -> E {
	move |err, _, state| {
		state.gtb.cancel_if(predicate);
		err
	}
}

/// See [`GreenBuilder::close`].
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_close<C: GreenCache, I>() -> impl Clone + Fn(I, SimpleSpan, &mut ParseState<C>) {
	|_, _, state| state.gtb.close()
}

/// See [`GreenBuilder::checkpoint`].
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_checkpoint<C: GreenCache>() -> impl Clone + Fn((), SimpleSpan, &mut ParseState<C>) {
	|_, _, state| {
		state.checkpoints.push(state.gtb.checkpoint());
	}
}

/// Combines [`GreenBuilder::open_at`] with a subsequent call to [`GreenBuilder::close`].
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_close_checkpoint<C: GreenCache, I>(
	kind: SyntaxKind,
) -> impl Clone + Fn(I, SimpleSpan, &mut ParseState<C>) {
	move |_, _, state| {
		state.gtb.open_at(
			state
				.checkpoints
				.pop()
				.expect("`gtb_close_checkpoint` had no checkpoints to pop."),
			kind,
		);

		state.gtb.close();
	}
}

/// Drops all children added since the most recent checkpoint.
/// Pass the return value of this to [`chumsky::Parser::map_err_with_state`].
pub fn gtb_cancel_checkpoint<C: GreenCache, E>(
) -> impl Clone + Fn(E, SimpleSpan, &mut ParseState<C>) -> E {
	|err, _, state| {
		state.gtb.cancel_checkpoint(
			state
				.checkpoints
				.pop()
				.expect("`gtb_cancel_checkpoint` had no checkpoints to pop."),
		);

		err
	}
}

/// Drops all children added since the most recent checkpoint.
/// Pass the return value of this to [`chumsky::Parser::map_with_state`].
pub fn gtb_cancel_checkpoint_if_none<C: GreenCache, I>(
) -> impl Clone + Fn(Option<I>, SimpleSpan, &mut ParseState<C>) {
	|opt, _, state| {
		if opt.is_none() {
			state.gtb.cancel_checkpoint(
				state
					.checkpoints
					.pop()
					.expect("`gtb_cancel_checkpoint` had no checkpoints to pop."),
			);
		}
	}
}
