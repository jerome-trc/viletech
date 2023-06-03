use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb,
	util::{builder::GreenCache, state::ParseState},
	zdoom::{
		decorate::Syn,
		lex::{Token, TokenStream},
		Extra,
	},
	ParseError,
};

use super::{
	common::*,
	expr,
	top::{const_def, enum_def},
};

pub fn actor_def<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::ActorDef.into(),
		primitive::group((
			comb::string_nc(Token::Ident, "actor", Syn::KwActor.into()),
			trivia_1plus(),
			actor_ident(),
			inherit_spec().or_not(),
			replaces_clause().or_not(),
			editor_number().or_not(),
			trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL.into()),
			actor_innard().repeated().collect::<()>(),
			comb::just_ts(Token::BraceR, Syn::BraceR.into()),
		)),
	)
	.boxed()
}

fn inherit_spec<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::InheritSpec.into(),
		primitive::group((
			trivia_0plus(),
			comb::just_ts(Token::Colon, Syn::Colon.into()),
			trivia_0plus(),
			actor_ident(),
		)),
	)
}

fn replaces_clause<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::ReplacesClause.into(),
		primitive::group((
			trivia_1plus(),
			comb::just_ts(Token::KwReplaces, Syn::KwReplaces.into()),
			trivia_1plus(),
			actor_ident(),
		)),
	)
}

fn editor_number<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::EditorNumber.into(),
		primitive::group((
			trivia_1plus(),
			comb::just_ts(Token::IntLit, Syn::IntLit.into()),
		)),
	)
}

pub fn actor_innard<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		trivia(),
		flag_setting(),
		const_def(),
		enum_def(),
		states_def(),
		property_settings(),
		user_var(),
	))
	.boxed()
}

pub fn flag_setting<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::FlagSetting.into(),
		primitive::group((
			primitive::choice((
				comb::just_ts(Token::Plus, Syn::Plus.into()),
				comb::just_ts(Token::Minus, Syn::Minus.into()),
			)),
			trivia_0plus(),
			ident_chain(),
		)),
	)
}

pub fn property_settings<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let part = primitive::choice((
		int_lit_negative(),
		float_lit_negative(),
		comb::just_ts(Token::IntLit, Syn::IntLit.into()),
		comb::just_ts(Token::FloatLit, Syn::FloatLit.into()),
		comb::just_ts(Token::StringLit, Syn::StringLit.into()),
		comb::just_ts(Token::NameLit, Syn::NameLit.into()),
		comb::just_ts(Token::KwTrue, Syn::KwTrue.into()),
		comb::just_ts(Token::KwFalse, Syn::KwFalse.into()),
		ident_chain(),
	));

	comb::node(
		Syn::PropertySettings.into(),
		primitive::group((
			part.clone(),
			comb::checkpointed(primitive::group((trivia_1plus(), part)))
				.repeated()
				.collect::<()>(),
		)),
	)
}

pub fn user_var<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::UserVar.into(),
		primitive::group((
			comb::just_ts(Token::KwVar, Syn::KwVar.into()),
			trivia_1plus(),
			primitive::choice((
				comb::just_ts(Token::KwInt, Syn::KwInt.into()),
				comb::just_ts(Token::KwFloat, Syn::KwFloat.into()),
			)),
			trivia_1plus(),
			comb::just_ts(Token::Ident, Syn::Ident.into()),
			trivia_0plus(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon.into()),
		)),
	)
}

// State machine definition ////////////////////////////////////////////////////

pub fn states_def<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::StatesDef.into(),
		primitive::group((
			comb::just_ts(Token::KwStates, Syn::KwStates.into()),
			trivia_0plus(),
			states_usage().or_not(),
			trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL.into()),
			primitive::choice((state_def(), state_label(), state_flow(), trivia()))
				.repeated()
				.collect::<()>(),
			comb::just_ts(Token::BraceR, Syn::BraceR.into()),
		)),
	)
	.boxed()
}

pub fn states_usage<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let single = primitive::choice((
		comb::string_nc(Token::Ident, "actor", Syn::Ident.into()),
		comb::string_nc(Token::Ident, "item", Syn::Ident.into()),
		comb::string_nc(Token::Ident, "overlay", Syn::Ident.into()),
		comb::string_nc(Token::Ident, "weapon", Syn::Ident.into()),
	));

	let rep = comb::checkpointed(primitive::group((
		comb::just_ts(Token::Comma, Syn::Comma.into()),
		trivia_0plus(),
		single.clone(),
		trivia_0plus(),
	)));

	comb::node(
		Syn::StatesUsage.into(),
		primitive::group((
			comb::just_ts(Token::ParenL, Syn::ParenL.into()),
			trivia_0plus(),
			single,
			trivia_0plus(),
			rep.repeated().collect::<()>(),
			comb::just_ts(Token::ParenR, Syn::ParenR.into()),
		)),
	)
}

pub fn state_label<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let name = primitive::one_of([Token::IntLit, Token::Ident])
		.repeated()
		.collect::<()>()
		.map_with_state(|(), span, state: &mut ParseState<C>| {
			state.gtb.token(Syn::Ident.into(), &state.source[span])
		});

	comb::node(
		Syn::StateLabel.into(),
		primitive::group((name, comb::just_ts(Token::Colon, Syn::Colon.into()))),
	)
}

pub fn state_flow<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let kw = primitive::choice((
		comb::just_ts(Token::KwStop, Syn::KwStop.into()),
		comb::string_nc(Token::Ident, "loop", Syn::KwLoop.into()),
		comb::string_nc(Token::Ident, "fail", Syn::KwFail.into()),
		comb::string_nc(Token::Ident, "wait", Syn::KwWait.into()),
	));

	let offset = comb::node(
		Syn::GotoOffset.into(),
		primitive::group((
			trivia_0plus(),
			comb::just_ts(Token::Plus, Syn::Plus.into()),
			trivia_0plus(),
			comb::just_ts(Token::IntLit, Syn::IntLit.into()),
		)),
	);

	let scope = comb::checkpointed(primitive::group((
		primitive::choice((
			comb::just_ts(Token::KwSuper, Syn::KwSuper.into()),
			comb::just_ts(Token::Ident, Syn::Ident.into()),
		)),
		trivia_0plus(),
		comb::just_ts(Token::Colon2, Syn::Colon2.into()),
		trivia_0plus(),
	)));

	let goto = comb::checkpointed(primitive::group((
		comb::just_ts(Token::KwGoto, Syn::KwGoto.into()),
		trivia_1plus(),
		scope.or_not(),
		ident_chain(),
		offset.or_not(),
	)))
	.map(|_| ());

	comb::node(Syn::StateFlow.into(), primitive::choice((kw, goto))).boxed()
}

pub fn state_def<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::StateDef.into(),
		primitive::group((
			state_sprite(),
			trivia_1line(),
			state_frames(),
			trivia_1line(),
			state_duration(),
			state_quals(),
			action_function().or_not(),
		)),
	)
	.boxed()
}

pub fn state_sprite<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let basic = primitive::one_of([Token::Ident, Token::IntLit])
		.repeated()
		.collect::<()>()
		.try_map_with_state(|(), span: logos::Span, state: &mut ParseState<C>| {
			if span.len() == 4 {
				state
					.gtb
					.token(Syn::StateSprite.into(), &state.source[span]);
				Ok(())
			} else {
				Err(ParseError::custom(
					span,
					"state sprite names must be exactly 4 characters long",
				))
			}
		});

	let hold = primitive::just(Token::StringLit).try_map_with_state(
		|_, span: logos::Span, state: &mut ParseState<C>| {
			if span.len() == 6 {
				state
					.gtb
					.token(Syn::StateSprite.into(), &state.source[span]);
				Ok(())
			} else {
				Err(ParseError::custom(
					span,
					"state sprite names must be exactly 4 characters long",
				))
			}
		},
	);

	primitive::choice((basic, hold))
}

pub fn state_frames<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	#[must_use]
	fn is_valid_quoted_char(c: char) -> bool {
		c.is_ascii_alphabetic() || c == '[' || c == ']' || c == '\\' || c == '#'
	}

	let unquoted = primitive::just(Token::Ident).try_map_with_state(
		|_, span: logos::Span, state: &mut ParseState<C>| {
			if !state.source[span.clone()].contains(|c: char| !c.is_ascii_alphabetic()) {
				state
					.gtb
					.token(Syn::StateFrames.into(), &state.source[span]);
				Ok(())
			} else {
				Err(ParseError::custom(
					span.clone(),
					format!("invalid frame character string `{}`", &state.source[span]),
				))
			}
		},
	);

	let quoted = primitive::just(Token::StringLit).try_map_with_state(
		|_, span: logos::Span, state: &mut ParseState<C>| {
			let inner = &state.source[(span.start + 1)..(span.end - 1)];

			if !inner.contains(|c: char| !is_valid_quoted_char(c)) {
				state
					.gtb
					.token(Syn::StateFrames.into(), &state.source[span]);
				Ok(())
			} else {
				Err(ParseError::custom(
					span.clone(),
					format!("invalid frame character string `{}`", &state.source[span]),
				))
			}
		},
	);

	primitive::choice((unquoted, quoted))
}

pub fn state_duration<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		int_lit_negative(),
		comb::just_ts(Token::IntLit, Syn::IntLit.into()),
		expr::call_expr(expr::expr()),
	))
	.boxed()
}

pub fn state_quals<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let light = comb::node(
		Syn::StateLight.into(),
		primitive::group((
			comb::just_ts(Token::KwLight, Syn::KwLight.into()),
			trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenL.into()),
			trivia_0plus(),
			primitive::choice((
				comb::just_ts(Token::StringLit, Syn::StringLit.into()),
				comb::just_ts(Token::NameLit, Syn::NameLit.into()),
			)),
			trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR.into()),
		)),
	);

	let offs_num = primitive::choice((
		int_lit_negative(),
		comb::just_ts(Token::IntLit, Syn::IntLit.into()),
	));

	let offset = comb::node(
		Syn::StateOffset.into(),
		primitive::group((
			comb::just_ts(Token::KwOffset, Syn::KwOffset.into()),
			trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenL.into()),
			trivia_0plus(),
			offs_num.clone(),
			trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma.into()),
			trivia_0plus(),
			offs_num,
			trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR.into()),
		)),
	);

	let qual = primitive::choice((
		comb::just_ts(Token::KwCanRaise, Syn::KwCanRaise.into()),
		comb::just_ts(Token::KwBright, Syn::KwBright.into()),
		comb::just_ts(Token::KwSlow, Syn::KwSlow.into()),
		comb::just_ts(Token::KwNoDelay, Syn::KwNoDelay.into()),
		comb::just_ts(Token::KwFast, Syn::KwFast.into()),
		light,
		offset,
	));

	comb::checkpointed(primitive::group((
		trivia_1line().repeated().at_least(1).collect::<()>(),
		qual,
	)))
	.repeated()
	.collect::<()>()
	.boxed()
}

pub fn action_function<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::ActionFunction.into(),
		primitive::group((
			trivia_1line().repeated().collect::<()>(),
			primitive::choice((
				expr::call_expr(expr::expr()),
				// TODO: Anonymous functions.
			)),
		)),
	)
	.boxed()
}

#[cfg(test)]
mod test {
	use rowan::ast::AstNode;

	use crate::{
		util::{builder::GreenCacheNoop, testing::*},
		zdoom::decorate::{ast, parse::file},
	};

	use super::*;

	#[test]
	fn smoke() {
		const SOURCE: &str = r#####"
aCtOr hangar : nuclearplant replaces toxinrefinery 10239 {
	enum {
		CMDCTRL,
		PHOBOSLAB = CMDCTRL,
		CENTPROC = 0,
		COMPSTAT = 9.9,
		PHOBOSANOMALY = false,
		MILBASE = "Yes, string literals are valid enum initializers in DECORATE!",
	}; // Floats and booleans too.

	CONST int DEIMOSANOMALY = 1234567890;

	var int dark_halls;
	var float hidingTheSecrets;

	ResetAllFlagsOrSomething
	Containment.Area
	+REFINERY
	DropItem "CMDCENTER" 255 1 PainSound "spawning/vats"

	States(Actor, overlay, ITEM, WeapoN) {
		Spawn: TNT1 A Random(1, 6)
		Wickedly:
			____ "#" 0
			goto super::Spawn.Something + 0
		Repent:
			3HA_ A 1 bright light('perfect')
			"####" B 6 canraise fast nodelay slow A_SpawnItemEx [rngtbl] (1, "??")
			"----" "]" -1 offset(-1, 1) light("sever")
			Loop
	}

	-TOWER.OF.BABEL
	Decal mysteryfortress
	ClassReference 'Pandemonium'

}
		"#####;

		let parser = file::<GreenCacheNoop>();

		let ptree = crate::parse(
			parser,
			None,
			Syn::Root.into(),
			SOURCE,
			Token::stream(SOURCE, None),
		);

		assert_no_errors(&ptree);

		let cursor = ptree.cursor::<Syn>();
		let toplevel = ast::TopLevel::cast(cursor.first_child().unwrap()).unwrap();

		let actordef = match toplevel {
			ast::TopLevel::ActorDef(inner) => inner,
			other => panic!("Expected `ActorDef`, found: {other:#?}"),
		};

		assert_eq!(actordef.name().text(), "hangar");

		assert_eq!(
			actordef
				.base_class()
				.expect("Actor definition has no base class.")
				.text(),
			"nuclearplant"
		);

		assert_eq!(
			actordef
				.replaced_class()
				.expect("Actor definition has no replacement clause.")
				.text(),
			"toxinrefinery"
		);

		assert_eq!(
			actordef
				.editor_number()
				.expect("Actor definition has no editor number.")
				.text()
				.parse::<u16>()
				.expect("Actor editor number is not a valid u16."),
			10239
		);

		let mut innards = actordef.innards();

		let _ = innards.next().unwrap().into_enumdef().unwrap();
		let constdef = innards.next().unwrap().into_constdef().unwrap();
		assert_eq!(constdef.name().text(), "DEIMOSANOMALY");
		assert_eq!(constdef.type_spec(), ast::ConstType::Int);

		let uservar1 = innards.next().unwrap().into_uservar().unwrap();
		assert_eq!(uservar1.name().text(), "dark_halls");
		assert_eq!(uservar1.type_spec(), ast::UserVarType::Int);
		let uservar2 = innards.next().unwrap().into_uservar().unwrap();
		assert_eq!(uservar2.name().text(), "hidingTheSecrets");
		assert_eq!(uservar2.type_spec(), ast::UserVarType::Float);

		let _ = innards.next().unwrap().into_propsettings().unwrap();

		let flag1 = innards.next().unwrap().into_flagsetting().unwrap();
		assert!(flag1.is_adding());
		assert_eq!(flag1.name().syntax().text(), "REFINERY");

		let _ = innards.next().unwrap().into_propsettings().unwrap();

		let statesdef = innards.next().unwrap().into_statesdef().unwrap();
		let mut usage_quals = statesdef.usage_quals().unwrap();

		assert_eq!(usage_quals.next().unwrap(), ast::StateUsage::Actor);
		assert_eq!(usage_quals.next().unwrap(), ast::StateUsage::Overlay);
		assert_eq!(usage_quals.next().unwrap(), ast::StateUsage::Item);
		assert_eq!(usage_quals.next().unwrap(), ast::StateUsage::Weapon);

		let mut state_items = statesdef.items();

		let label1 = state_items.next().unwrap().into_label().unwrap();
		assert_eq!(label1.token().text(), "Spawn");

		let state1 = state_items.next().unwrap().into_state().unwrap();

		assert_eq!(state1.sprite().text(), "TNT1");
		assert_eq!(state1.frames().text(), "A");

		let state1_dur = state1.duration().into_node().unwrap();
		assert_eq!(
			ast::ExprCall::cast(state1_dur).unwrap().name().text(),
			"Random"
		);

		let label2 = state_items.next().unwrap().into_label().unwrap();
		assert_eq!(label2.token().text(), "Wickedly");

		let _state2 = state_items.next().unwrap().into_state().unwrap();
		let change1 = state_items.next().unwrap().into_flow().unwrap();
		match change1 {
			ast::StateFlow::Goto {
				target,
				offset,
				scope,
			} => {
				assert_eq!(target.syntax().text(), "Spawn.Something");
				assert_eq!(offset.unwrap(), 0);
				assert_eq!(scope.unwrap().kind(), Syn::KwSuper);
			}
			other => panic!("Expected `StateChange::Goto`, found: {other:#?}"),
		}

		let _ = state_items.next().unwrap().into_label().unwrap();
		let state3 = state_items.next().unwrap().into_state().unwrap();
		assert_eq!(state3.light().unwrap().text(), "'perfect'");

		let state4 = state_items.next().unwrap().into_state().unwrap();
		let mut state4_quals = state4.qualifiers();

		assert!(matches!(
			state4_quals.next().unwrap(),
			ast::StateQual::CanRaise
		));
		assert!(matches!(state4_quals.next().unwrap(), ast::StateQual::Fast));
		assert!(matches!(
			state4_quals.next().unwrap(),
			ast::StateQual::NoDelay
		));
		assert!(matches!(state4_quals.next().unwrap(), ast::StateQual::Slow));
	}
}
