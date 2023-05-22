use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb,
	util::{builder::GreenCache, state::*},
	zdoom::{decorate::Syn, lexer::*},
	Extra,
};

use super::common::{self, wsp_ext};

pub(super) fn expr<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	// Parsing DECORATE expressions is tricky. In the reference implementation,
	// a function call does not require a parenthesis-delimited argument list,
	// making them ambiguous with all other identifiers. As such, it uses a special
	// variation on `expr_call` that requires the argument list; all other identifiers
	// are assumed to be atoms, and must be handled by DoomFront's callers.
	chumsky::recursive::recursive(|expr| {
		let atom = primitive::choice((
			comb::kw_nc("false")
				.map_with_state(gtb_open_with(Syn::Literal.into(), Syn::LitFalse.into())),
			comb::kw_nc("true")
				.map_with_state(gtb_open_with(Syn::Literal.into(), Syn::LitTrue.into())),
			common::ident().map_with_state(gtb_open_with(Syn::ExprIdent.into(), Syn::Ident.into())),
			lit_float().map_with_state(gtb_open_with(Syn::Literal.into(), Syn::LitFloat.into())),
			lit_int().map_with_state(gtb_open_with(Syn::Literal.into(), Syn::LitInt.into())),
			lit_name().map_with_state(gtb_open_with(Syn::Literal.into(), Syn::LitName.into())),
			lit_string().map_with_state(gtb_open_with(Syn::Literal.into(), Syn::LitString.into())),
		));

		let call = primitive::group((
			common::ident().map_with_state(gtb_open_with(Syn::ExprCall.into(), Syn::Ident.into())),
			wsp_ext().repeated().collect::<()>(),
			rng_spec().or_not(),
			wsp_ext().repeated().collect::<()>(),
			arg_list(expr),
		))
		.map_with_state(gtb_close())
		.map_err_with_state(gtb_cancel(Syn::ExprCall.into()));

		// TODO: DECORATE's imperative component.

		primitive::choice((atom, call))
	})
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel_if(|kind| {
		kind == Syn::ExprIdent.into() || kind == Syn::Literal.into() || kind == Syn::ExprCall.into()
	}))
}

pub(super) fn expr_call<'i, C: 'i + GreenCache>(
	expr: impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone,
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	primitive::group((
		common::ident().map_with_state(gtb_open_with(Syn::ExprCall.into(), Syn::Ident.into())),
		wsp_ext().repeated().collect::<()>(),
		rng_spec().or_not(),
		wsp_ext().repeated().collect::<()>(),
		arg_list(expr).or_not(),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::ExprCall.into()))
}

/// Can be part of a function call between the identifier and argument list.
fn rng_spec<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	primitive::group((
		primitive::just("[")
			.map_with_state(gtb_open_with(Syn::RngSpec.into(), Syn::BracketL.into())),
		common::ident().map_with_state(gtb_token(Syn::Ident.into())),
		primitive::just("]").map_with_state(gtb_token(Syn::BracketR.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::RngSpec.into()))
}

/// Includes the delimiting parentheses.
fn arg_list<'i, C: 'i + GreenCache>(
	expr: impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone,
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	let arg = primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::Argument.into())),
		expr,
	))
	.map_with_state(gtb_close())
	.map_with_state(gtb_cancel(Syn::Argument.into()));

	let rep = primitive::group((
		primitive::just(",").map_with_state(gtb_token(Syn::Comma.into())),
		wsp_ext().repeated().collect::<()>(),
		arg.clone(),
		wsp_ext().repeated().collect::<()>(),
	));

	primitive::group((
		primitive::just("(").map_with_state(gtb_open_with(Syn::ArgList.into(), Syn::ParenL.into())),
		wsp_ext().repeated().collect::<()>(),
		arg,
		wsp_ext().repeated().collect::<()>(),
		rep.repeated().collect::<()>(),
		primitive::just(")").map_with_state(gtb_token(Syn::ParenR.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::ArgList.into()))
}
