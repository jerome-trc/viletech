mod actor;
mod common;
mod expr;
mod top;

use chumsky::{primitive, IterParser, Parser};

use crate::{
	util::builder::GreenCache,
	zdoom::{lex::TokenStream, Extra},
};

pub use self::{actor::*, common::*, expr::*, top::*};

pub fn file<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let ret = primitive::choice((
		trivia(),
		actor_def(),
		include_directive(),
		const_def(),
		enum_def(),
	))
	.repeated()
	.collect::<()>();

	#[cfg(any(debug_assertions, test))]
	{
		ret.boxed()
	}
	#[cfg(not(any(debug_assertions, test)))]
	{
		ret
	}
}
