use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb,
	util::{builder::GreenCache, state::*},
	zdoom::{self, decorate::Syn, lexer::*},
	Extra,
};

use super::{
	common::*,
	expr::*,
	top::{const_def, enum_def},
};

pub(super) fn actor_def<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		comb::kw_nc("actor")
			.map_with_state(gtb_open_with(Syn::ActorDef.into(), Syn::KwActor.into())),
		wsp_ext().repeated().at_least(1).collect::<()>(),
		actor_ident().map_with_state(gtb_token(Syn::Ident.into())),
		inherit_spec().or_not(),
		replaces_clause().or_not(),
		editor_number().or_not(),
		wsp_ext().repeated().collect::<()>(),
		primitive::just("{").map_with_state(gtb_token(Syn::BraceL.into())),
		actor_innards().repeated().collect::<()>(),
		primitive::just("}").map_with_state(gtb_token(Syn::BraceR.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::ActorDef.into()))
}

fn inherit_spec<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		primitive::empty().map_with_state(gtb_checkpoint()),
		wsp_ext().repeated().collect::<()>(),
		primitive::just(":").map_with_state(gtb_token(Syn::Colon.into())),
		wsp_ext().repeated().collect::<()>(),
		actor_ident().map_with_state(gtb_token(Syn::Ident.into())),
	))
	.map_with_state(gtb_close_checkpoint(Syn::InheritSpec.into()))
	.map_err_with_state(gtb_cancel_checkpoint())
}

/// `whitespace+ 'replaces' whitespace+ ident`
fn replaces_clause<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		primitive::empty().map_with_state(gtb_checkpoint()),
		wsp_ext().repeated().at_least(1).collect::<()>(),
		comb::kw_nc("replaces").map_with_state(gtb_token(Syn::KwReplaces.into())),
		wsp_ext().repeated().at_least(1).collect::<()>(),
		actor_ident().map_with_state(gtb_token(Syn::Ident.into())),
	))
	.map_with_state(gtb_close_checkpoint(Syn::ReplacesClause.into()))
	.map_err_with_state(gtb_cancel_checkpoint())
}

fn editor_number<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		primitive::empty().map_with_state(gtb_checkpoint()),
		wsp_ext().repeated().at_least(1).collect::<()>(),
		zdoom::lexer::lit_int().map_with_state(gtb_token(Syn::LitInt.into())),
	))
	.map_with_state(gtb_close_checkpoint(Syn::EditorNumber.into()))
	.map_err_with_state(gtb_cancel_checkpoint())
}

fn actor_innards<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::choice((
		wsp_ext(),
		flag_setting(),
		const_def(),
		enum_def(),
		states_def(),
		property_settings(),
	))
}

fn flag_setting<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		primitive::one_of(['+', '-']).map_with_state(|c, _, state: &mut ParseState<C>| {
			state.gtb.open(Syn::FlagSetting.into());

			match c {
				'+' => state.gtb.token(Syn::Plus.into(), "+"),
				'-' => state.gtb.token(Syn::Minus.into(), "-"),
				_ => unreachable!(),
			}
		}),
		wsp_ext().repeated().collect::<()>(),
		ident_chain(),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::FlagSetting.into()))
}

fn property_settings<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	let part = primitive::choice((
		lit_int_negative().map_with_state(gtb_token(Syn::LitInt.into())),
		lit_float_negative().map_with_state(gtb_token(Syn::LitFloat.into())),
		lit_int().map_with_state(gtb_token(Syn::LitInt.into())),
		lit_float().map_with_state(gtb_token(Syn::LitFloat.into())),
		lit_string().map_with_state(gtb_token(Syn::LitString.into())),
		lit_name().map_with_state(gtb_token(Syn::LitName.into())),
		comb::kw_nc("true").map_with_state(gtb_token(Syn::LitTrue.into())),
		comb::kw_nc("false").map_with_state(gtb_token(Syn::LitFalse.into())),
		ident_chain(),
	));

	primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::PropertySettings.into())),
		part.clone(),
		primitive::group((
			primitive::empty().map_with_state(gtb_checkpoint()),
			wsp_ext().repeated().at_least(1).collect::<()>(),
			part,
		))
		.map_err_with_state(gtb_cancel_checkpoint())
		.repeated()
		.collect::<()>(),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::PropertySettings.into()))
}

// State machine definition ////////////////////////////////////////////////////

fn states_def<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		comb::kw_nc("states")
			.map_with_state(gtb_open_with(Syn::StatesDef.into(), Syn::KwStates.into())),
		wsp_ext().repeated().collect::<()>(),
		states_usage().or_not(),
		wsp_ext().repeated().collect::<()>(),
		primitive::just("{").map_with_state(gtb_token(Syn::BraceL.into())),
		primitive::choice((
			state_def(),
			state_label(),
			state_change(),
			wsp_ext().repeated().at_least(1).collect::<()>(),
		))
		.repeated()
		.collect::<()>(),
		primitive::just("}").map_with_state(gtb_token(Syn::BraceR.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::StatesDef.into()))
}

fn states_usage<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		primitive::just("(")
			.map_with_state(gtb_open_with(Syn::StatesUsage.into(), Syn::ParenL.into())),
		primitive::choice((
			comb::kw_nc("actor").map_with_state(gtb_token(Syn::StatesUsageActor.into())),
			comb::kw_nc("item").map_with_state(gtb_token(Syn::StatesUsageItem.into())),
			comb::kw_nc("overlay").map_with_state(gtb_token(Syn::StatesUsageOverlay.into())),
			comb::kw_nc("weapon").map_with_state(gtb_token(Syn::StatesUsageWeapon.into())),
		)),
		primitive::just(")").map_with_state(gtb_token(Syn::ParenR.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::StatesUsage.into()))
}

fn state_label<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::StateLabel.into())),
		actor_ident().map_with_state(gtb_token(Syn::Ident.into())),
		primitive::just(":").map_with_state(gtb_token(Syn::Colon.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::StateLabel.into()))
}

fn state_change<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	let kw = primitive::choice((
		comb::kw_nc("fail").map_with_state(gtb_token(Syn::KwFail.into())),
		comb::kw_nc("loop").map_with_state(gtb_token(Syn::KwLoop.into())),
		comb::kw_nc("stop").map_with_state(gtb_token(Syn::KwStop.into())),
		comb::kw_nc("wait").map_with_state(gtb_token(Syn::KwWait.into())),
	));

	let offset = primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::GotoOffset.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::just("+").map_with_state(gtb_token(Syn::Plus.into())),
		wsp_ext().repeated().collect::<()>(),
		lit_int().map_with_state(gtb_token(Syn::LitInt.into())),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::GotoOffset.into()));

	let sup = primitive::group((
		primitive::empty().map_with_state(gtb_checkpoint()),
		comb::kw_nc("super").map_with_state(gtb_token(Syn::Ident.into())),
		primitive::just("::").map_with_state(gtb_token(Syn::Colon2.into())),
	))
	.map_err_with_state(gtb_cancel_checkpoint());

	let goto = primitive::group((
		comb::kw_nc("goto").map_with_state(gtb_token(Syn::KwGoto.into())),
		wsp_ext().repeated().at_least(1).collect::<()>(),
		sup.or_not(),
		ident_chain(),
		offset.or_not(),
	));

	primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::StateChange.into())),
		primitive::choice((kw, goto.map(|_| ()))),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::StateChange.into()))
}

fn state_def<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::StateDef.into())),
		state_sprite(),
		wsp_1line().repeated().at_least(1).collect::<()>(),
		state_frames(),
		wsp_1line().repeated().at_least(1).collect::<()>(),
		state_duration(),
		state_quals(),
		action_function().or_not(),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::StateDef.into()))
}

fn state_sprite<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	let alphanum = primitive::choice((
		comb::ascii_letter(),
		comb::dec_digit(),
		primitive::just('_'),
	))
	.repeated()
	.exactly(4)
	.slice();

	primitive::choice((
		alphanum.map_with_state(gtb_token(Syn::StateSprite.into())),
		primitive::just("\"####\"").map_with_state(gtb_token(Syn::StateSprite.into())),
		primitive::just("\"----\"").map_with_state(gtb_token(Syn::StateSprite.into())),
	))
}

fn state_frames<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	let alpha = primitive::one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
		.repeated()
		.at_least(1)
		.slice();

	primitive::choice((
		alpha.map_with_state(gtb_token(Syn::StateFrames.into())),
		zdoom::lexer::lit_string().map_with_state(gtb_token(Syn::LitString.into())),
	))
}

fn state_duration<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::choice((
		lit_int().map_with_state(gtb_token(Syn::LitInt.into())),
		lit_int_negative().map_with_state(gtb_token(Syn::LitInt.into())),
		expr_call(expr()),
	))
}

fn state_quals<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	let light = primitive::group((
		comb::kw_nc("light")
			.map_with_state(gtb_open_with(Syn::StateLight.into(), Syn::KwLight.into())),
		primitive::just("(").map_with_state(gtb_token(Syn::ParenL.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::choice((
			lit_string().map_with_state(gtb_token(Syn::LitString.into())),
			lit_name().map_with_state(gtb_token(Syn::LitName.into())),
		)),
		wsp_ext().repeated().collect::<()>(),
		primitive::just(")").map_with_state(gtb_token(Syn::ParenR.into())),
	))
	.map_with_state(gtb_close())
	.map_with_state(gtb_cancel(Syn::StateLight.into()));

	let offset = primitive::group((
		comb::kw_nc("offset")
			.map_with_state(gtb_open_with(Syn::StateOffset.into(), Syn::KwOffset.into())),
		primitive::just("(").map_with_state(gtb_token(Syn::ParenL.into())),
		wsp_ext().repeated().collect::<()>(),
		primitive::choice((
			lit_int().map_with_state(gtb_token(Syn::LitInt.into())),
			lit_int_negative().map_with_state(gtb_token(Syn::LitInt.into())),
		)),
		primitive::just(","),
		wsp_ext().repeated().collect::<()>(),
		primitive::choice((
			lit_int().map_with_state(gtb_token(Syn::LitInt.into())),
			lit_int_negative().map_with_state(gtb_token(Syn::LitInt.into())),
		)),
		primitive::just(")").map_with_state(gtb_token(Syn::ParenR.into())),
	))
	.map_with_state(gtb_close())
	.map_with_state(gtb_cancel(Syn::StateOffset.into()));

	let qual = primitive::choice((
		comb::kw_nc("bright").map_with_state(gtb_token(Syn::KwBright.into())),
		comb::kw_nc("canraise").map_with_state(gtb_token(Syn::KwCanRaise.into())),
		comb::kw_nc("fast").map_with_state(gtb_token(Syn::KwFast.into())),
		comb::kw_nc("nodelay").map_with_state(gtb_token(Syn::KwNoDelay.into())),
		comb::kw_nc("slow").map_with_state(gtb_token(Syn::KwSlow.into())),
		// Parameterized ///////////////////////////////////////////////////////
		light,
		offset,
	));

	primitive::group((
		primitive::empty().map_with_state(gtb_checkpoint()),
		wsp_1line().repeated().at_least(1).collect::<()>(),
		qual,
	))
	.map_err_with_state(gtb_cancel_checkpoint())
	.repeated()
	.collect::<()>()
}

fn action_function<'i, C: 'i + GreenCache>() -> impl Parser<'i, &'i str, (), Extra<'i, C>> {
	primitive::group((
		primitive::empty().map_with_state(gtb_open(Syn::ActionFunction.into())),
		wsp_1line().repeated().collect::<()>(),
		primitive::choice((expr_call(expr()) /* TODO: Anonymous functions. */,)),
	))
	.map_with_state(gtb_close())
	.map_err_with_state(gtb_cancel(Syn::ActionFunction.into()))
}
