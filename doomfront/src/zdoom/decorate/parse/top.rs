//! Non-actor top-level elements: symbolic constants, enums, preprocessor directives.

use chumsky::{primitive, text, IterParser, Parser};

use crate::{
	comb,
	util::{builder::GreenCache, state::*},
	zdoom::{decorate::Syn, lexer::*},
	Extra,
};

use super::{common::*, expr};

pub(super) fn const_def<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		comb::kw_nc("const")
			.map_with_state(gtb_open_with(Syn::ConstDef.into(), Syn::KwConst.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::choice((
			comb::kw_nc("fixed").map_with_state(gtb_token(Syn::KwFixed.into())),
			comb::kw_nc("float").map_with_state(gtb_token(Syn::KwFloat.into())),
			comb::kw_nc("int").map_with_state(gtb_token(Syn::KwInt.into())),
		)),
		wsp_ext().repeated().collect::<()>(),
		text::ident().map_with_state(gtb_token(Syn::Ident.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::just("=").map_with_state(gtb_token(Syn::Eq.into())),
		wsp_ext().repeated().collect::<()>(),
		expr::expr(),
		wsp_ext().repeated().collect::<()>(),
		primitive::just(";").map_with_state(gtb_token(Syn::Semicolon.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::ConstDef.into()))
}

pub(super) fn enum_def<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		comb::kw_nc("enum").map_with_state(gtb_open_with(Syn::EnumDef.into(), Syn::KwEnum.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::just("{").map_with_state(gtb_token(Syn::BraceL.into())),
		wsp_ext().repeated().collect::<()>(),
		enum_variants(),
		wsp_ext().repeated().collect::<()>(),
		primitive::just("}").map_with_state(gtb_token(Syn::BraceR.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::just(";").map_with_state(gtb_token(Syn::Semicolon.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::EnumDef.into()))
}

fn enum_variants<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	let init = primitive::group((
		text::ident().map_with_state(gtb_open_with(Syn::EnumVariant.into(), Syn::Ident.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::just("=").map_with_state(gtb_token(Syn::Eq.into())),
		wsp_ext().repeated().collect::<()>(),
		expr::expr(),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::EnumVariant.into()));

	let uninit =
		text::ident().map_with_state(gtb_open_close(Syn::EnumVariant.into(), Syn::Ident.into()));

	let variant = primitive::choice((init, uninit));

	let successive = primitive::group((
		primitive::empty().map_with_state(gtb_checkpoint()),
		wsp_ext().repeated().collect::<()>(),
		primitive::just(",").map_with_state(gtb_token(Syn::Comma.into())),
		wsp_ext().repeated().collect::<()>(),
		variant.clone(),
	))
	.map_err_with_state(gtb_cancel_checkpoint())
	.repeated()
	.collect::<()>();

	primitive::group((
		variant,
		successive,
		primitive::just(",")
			.or_not()
			.map_with_state(gtb_token_opt(Syn::Comma.into())),
	))
	.map(|_| ())
}

pub(super) fn include_directive<'i, C: 'i + GreenCache>(
) -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		comb::kw_nc("#include").map_with_state(gtb_open_with(
			Syn::IncludeDirective.into(),
			Syn::PreprocInclude.into(),
		)),
		wsp_1line().repeated().collect::<()>(),
		lit_string().map_with_state(gtb_token(Syn::LitString.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::IncludeDirective.into()))
}
