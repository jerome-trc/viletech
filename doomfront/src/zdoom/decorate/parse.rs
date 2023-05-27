mod actor;
mod common;
mod expr;
mod top;

use chumsky::{primitive, IterParser, Parser};

use crate::{
	util::{builder::GreenCache, state::ParseState},
	ParseTree,
};

use super::Syn;

use self::{actor::*, common::*, top::*};

#[must_use]
pub fn parse<'i, C: 'i + GreenCache>(source: &'i str, cache: Option<C>) -> Option<ParseTree<'i>> {
	let parser = primitive::choice((
		wsp_ext(),
		actor_def(),
		include_directive(),
		const_def(),
		enum_def(),
	))
	.repeated()
	.collect::<()>();

	let mut state = ParseState::new(cache);

	state.gtb.open(Syn::Root.into());

	let (output, errors) = parser
		.parse_with_state(source, &mut state)
		.into_output_errors();

	output.map(|_| {
		state.gtb.close();

		ParseTree {
			root: state.gtb.finish(),
			errors,
		}
	})
}
