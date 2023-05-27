//! Various combinators which are broadly applicable elsewhere.

use doomfront::{
	chumsky::{primitive, text, IterParser, Parser},
	comb,
	util::{builder::GreenCache, state::*},
	Extra,
};

use crate::vzs::parse::expr::*;

use super::Syn;

/// Builds a [`Syn::Annotation`] node.
pub(super) fn annotation<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	primitive::group((
		primitive::just("#")
			.map_with_state(gtb_open_with(Syn::Annotation.into(), Syn::Pound.into())),
		primitive::just("!")
			.or_not()
			.map_with_state(gtb_token_opt(Syn::Bang.into())),
		primitive::just("[").map_with_state(gtb_token(Syn::BracketL.into())),
		wsp_ext().repeated().collect::<()>(),
		resolver(),
		wsp_ext().repeated().collect::<()>(),
		arg_list().or_not(),
		wsp_ext().repeated().collect::<()>(),
		primitive::just("]").map_with_state(gtb_token(Syn::BracketR.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::Annotation.into()))
}

pub(super) fn any_kw<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	primitive::choice((
		primitive::just("break"),
		primitive::just("ceval"),
		primitive::just("continue"),
		primitive::just("const"),
		primitive::just("do"),
		primitive::just("else"),
		primitive::just("false"),
		primitive::just("for"),
		primitive::just("if"),
		primitive::just("let"),
		primitive::just("return"),
		primitive::just("switch"),
		primitive::just("true"),
		primitive::just("until"),
		primitive::just("while"),
	))
}

/// Includes delimiting parentheses. Builds a [`Syn::ArgList`] node.
pub(super) fn arg_list<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	let anon = primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::Argument.into())),
		expr(),
	))
	.map_with_state(gtb_close())
	.map_with_state(gtb_cancel(Syn::Argument.into()));

	let labelled = primitive::group((
		ident().map_with_state(gtb_open_with(Syn::Argument.into(), Syn::Ident.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::just(":").map_with_state(gtb_token(Syn::Colon.into())),
		wsp_ext().repeated().collect::<()>(),
		expr(),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::Argument.into()));

	let rep = primitive::group((
		primitive::just(",").map_with_state(gtb_token(Syn::Comma.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::choice((labelled.clone(), anon.clone())),
		wsp_ext().repeated().collect::<()>(),
	));

	primitive::group((
		primitive::just("(").map_with_state(gtb_open_with(Syn::ArgList.into(), Syn::ParenL.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::choice((labelled, anon)).or_not(),
		wsp_ext().repeated().collect::<()>(),
		rep.repeated().collect::<()>(),
		primitive::just(")").map_with_state(gtb_token(Syn::ParenR.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::ArgList.into()))
}

/// Shorthand for `chumsky::text::ident().and_is(any_kw().not())`.
pub(super) fn ident<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, &'i str, Extra<'i, C>> + Clone {
	text::ident().and_is(any_kw().not()).slice()
}

/// Builds a [`Syn::Name`] node.
pub(super) fn name<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	ident().map_with_state(gtb_open_close(Syn::Name.into(), Syn::Ident.into()))
}

/// Builds a [`Syn::Resolver`] node.
pub(super) fn resolver<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone {
	let delimiter = primitive::just("::").map_with_state(gtb_token(Syn::Colon2.into()));

	let rep = primitive::group((delimiter.clone(), name()));

	primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::Resolver.into())),
		delimiter.or_not(),
		name(),
		rep.repeated().collect::<()>(),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::Resolver.into()))
}

/// Remember that this matches either a heterogenous whitespace span or a comment,
/// so to deal with both in one span requires repetition.
/// Builds [`Syn::Comment`] or [`Syn::Whitespace`] tokens.
pub(super) fn wsp_ext<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> + Clone
{
	primitive::choice((
		comb::cpp_comment().map_with_state(gtb_token(Syn::Comment.into())),
		comb::c_comment().map_with_state(gtb_token(Syn::Comment.into())),
		comb::wsp().map_with_state(gtb_token(Syn::Whitespace.into())),
	))
}
