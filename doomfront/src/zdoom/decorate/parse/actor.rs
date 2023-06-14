use chumsky::{primitive, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{decorate::Syn, Token},
	GreenElement, ParseError, ParseState,
};

use super::{common::*, expr, top::*};

/// The returned parser emits a [`Syn::ActorDef`] node.
pub fn actor_def<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		comb::string_nc(Token::Ident, "actor", Syn::KwActor),
		trivia_1plus(),
		actor_ident(),
		inherit_spec().or_not(),
		replaces_clause().or_not(),
		editor_number().or_not(),
		trivia_0plus(),
		comb::just_ts(Token::BraceL, Syn::BraceL),
		actor_innard().repeated().collect::<Vec<_>>(),
		comb::just_ts(Token::BraceR, Syn::BraceR),
	))
	.map(|group| coalesce_node(group, Syn::ActorDef))
	.boxed()
}

/// The returned parser emits a [`Syn::InheritSpec`] node.
fn inherit_spec<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		trivia_0plus(),
		comb::just_ts(Token::Colon, Syn::Colon),
		trivia_0plus(),
		actor_ident(),
	))
	.map(|group| coalesce_node(group, Syn::InheritSpec))
}

/// The returned parser emits a [`Syn::ReplacesClause`] node.
fn replaces_clause<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		trivia_1plus(),
		comb::just_ts(Token::KwReplaces, Syn::KwReplaces),
		trivia_1plus(),
		actor_ident(),
	))
	.map(|group| coalesce_node(group, Syn::ReplacesClause))
}

/// The returned parser emits a [`Syn::EditorNumber`] node.
fn editor_number<'i>() -> parser_t!(GreenNode) {
	primitive::group((trivia_1plus(), comb::just_ts(Token::IntLit, Syn::IntLit)))
		.map(|group| coalesce_node(group, Syn::EditorNumber))
}

pub fn actor_innard<'i>() -> parser_t!(GreenElement) {
	primitive::choice((
		trivia(),
		flag_setting().map(|gnode| gnode.into()),
		const_def().map(|gnode| gnode.into()),
		enum_def().map(|gnode| gnode.into()),
		states_def().map(|gnode| gnode.into()),
		user_var().map(|gnode| gnode.into()),
		property_settings().map(|gnode| gnode.into()),
	))
	.boxed()
}

/// The returned parser emits a [`Syn::FlagSetting`] node.
pub fn flag_setting<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		primitive::choice((
			comb::just_ts(Token::Plus, Syn::Plus),
			comb::just_ts(Token::Minus, Syn::Minus),
		)),
		trivia_0plus(),
		ident_chain(),
	))
	.map(|group| coalesce_node(group, Syn::FlagSetting))
}

/// The returned parser emits a [`Syn::PropertySettings`] node.
pub fn property_settings<'i>() -> parser_t!(GreenNode) {
	let parenthesized = primitive::group((
		comb::just_ts(Token::ParenL, Syn::ParenL),
		trivia_0plus(),
		expr::expr(),
		trivia_0plus(),
		comb::just_ts(Token::ParenR, Syn::ParenR),
	))
	.map(coalesce_vec);

	let part = primitive::choice((
		flag_setting().map(coalesce_vec),
		parenthesized,
		int_lit_negative().map(coalesce_vec),
		float_lit_negative().map(coalesce_vec),
		expr::expr().map(coalesce_vec),
		ident_chain().map(coalesce_vec),
	));

	let delim = primitive::choice((
		primitive::group((
			trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			trivia_0plus(),
		))
		.map(coalesce_vec),
		trivia_1plus(),
	));

	primitive::group((
		primitive::choice((flag_setting(), ident_chain())),
		primitive::group((delim, part))
			.repeated()
			.collect::<Vec<_>>(),
	))
	.map(|group| coalesce_node(group, Syn::PropertySettings))
}

/// The returned parser emits a [`Syn::UserVar`] node.
pub fn user_var<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		comb::just_ts(Token::KwVar, Syn::KwVar),
		trivia_1plus(),
		primitive::choice((
			comb::just_ts(Token::KwInt, Syn::KwInt),
			comb::just_ts(Token::KwFloat, Syn::KwFloat),
		)),
		trivia_1plus(),
		comb::just_ts(Token::Ident, Syn::Ident),
		trivia_0plus(),
		comb::just_ts(Token::Semicolon, Syn::Semicolon),
	))
	.map(|group| coalesce_node(group, Syn::UserVar))
}

// State machine definition ////////////////////////////////////////////////////

/// The returned parser emits a [`Syn::StatesDef`] node.
pub fn states_def<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		comb::just_ts(Token::KwStates, Syn::KwStates),
		trivia_0plus(),
		states_usage().or_not(),
		trivia_0plus(),
		comb::just_ts(Token::BraceL, Syn::BraceL),
		primitive::choice((
			state_def().map(GreenElement::from),
			state_label().map(GreenElement::from),
			state_flow().map(GreenElement::from),
			trivia(),
		))
		.repeated()
		.collect::<Vec<_>>(),
		comb::just_ts(Token::BraceR, Syn::BraceR),
	))
	.map(|group| coalesce_node(group, Syn::StatesDef))
	.boxed()
}

/// The returned parser emits a [`Syn::StatesUsage`] node.
pub fn states_usage<'i>() -> parser_t!(GreenNode) {
	let single = primitive::choice((
		comb::string_nc(Token::Ident, "actor", Syn::Ident),
		comb::string_nc(Token::Ident, "item", Syn::Ident),
		comb::string_nc(Token::Ident, "overlay", Syn::Ident),
		comb::string_nc(Token::Ident, "weapon", Syn::Ident),
	));

	let rep = primitive::group((
		comb::just_ts(Token::Comma, Syn::Comma),
		trivia_0plus(),
		single.clone(),
		trivia_0plus(),
	))
	.map(coalesce_vec);

	primitive::group((
		comb::just_ts(Token::ParenL, Syn::ParenL),
		trivia_0plus(),
		single,
		trivia_0plus(),
		rep.repeated().collect::<Vec<_>>(),
		comb::just_ts(Token::ParenR, Syn::ParenR),
	))
	.map(|group| coalesce_node(group, Syn::StatesUsage))
}

/// The returned parser emits a [`Syn::StateLabel`] node.
pub fn state_label<'i>() -> parser_t!(GreenNode) {
	let part = primitive::any()
		.filter(|t| {
			if matches!(t, Token::KwFail | Token::KwStop | Token::KwWait) {
				return false;
			}

			if t.is_keyword() {
				return true;
			}

			matches!(t, Token::Ident | Token::IntLit)
		})
		.repeated()
		.at_least(1)
		.collect::<()>()
		.map_with_state(|(), mut span: logos::Span, state: &mut ParseState| {
			if span.start > span.end {
				// FIXME: Possible Chumsky bug! Not a lexer issue; that's working fine.
				std::mem::swap(&mut span.start, &mut span.end);
			}

			GreenToken::new(Syn::Ident.into(), &state.source[span])
		});

	let name = primitive::group((
		part,
		(primitive::group((
			trivia_0plus(),
			comb::just_ts(Token::Dot, Syn::Dot),
			trivia_0plus(),
			part,
		)))
		.repeated()
		.collect::<Vec<_>>(),
	));

	primitive::group((
		name,
		trivia_0plus(),
		comb::just_ts(Token::Colon, Syn::Colon),
	))
	.map(|group| coalesce_node(group, Syn::StateLabel))
}

/// The returned parser emits a [`Syn::StateFlow`] node.
pub fn state_flow<'i>() -> parser_t!(GreenNode) {
	let kw = primitive::choice((
		comb::just_ts(Token::KwStop, Syn::KwStop),
		comb::just_ts(Token::KwLoop, Syn::KwLoop),
		comb::just_ts(Token::KwFail, Syn::KwFail),
		comb::just_ts(Token::KwWait, Syn::KwWait),
	))
	.map(|group| coalesce_node(group, Syn::StateFlow));

	let offset = primitive::group((
		trivia_0plus(),
		comb::just_ts(Token::Plus, Syn::Plus),
		trivia_0plus(),
		comb::just_ts(Token::IntLit, Syn::IntLit),
	))
	.map(|group| coalesce_node(group, Syn::GotoOffset));

	let scope = primitive::group((
		primitive::choice((
			comb::just_ts(Token::KwSuper, Syn::KwSuper),
			comb::just_ts(Token::Ident, Syn::Ident),
		)),
		trivia_0plus(),
		comb::just_ts(Token::Colon2, Syn::Colon2),
		trivia_0plus(),
	));

	let goto = primitive::group((
		comb::just_ts(Token::KwGoto, Syn::KwGoto),
		trivia_1plus(),
		scope.or_not(),
		ident_chain(),
		offset.or_not(),
	))
	.map(|group| coalesce_node(group, Syn::StateFlow));

	primitive::choice((kw, goto))
}

/// The returned parser emits a [`Syn::StateDef`] node.
pub fn state_def<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		state_sprite(),
		trivia_1line(),
		state_frames(),
		trivia_1line(),
		state_duration(),
		state_quals(),
		action_function().or_not(),
	))
	.map(|group| coalesce_node(group, Syn::StateDef))
}

/// The returned parser emits a [`Syn::StateSprite`] token.
pub fn state_sprite<'i>() -> parser_t!(GreenToken) {
	let basic = primitive::any()
		.filter(|t: &Token| {
			if !(*t == Token::Ident || *t == Token::IntLit || t.is_keyword()) {
				return false;
			}

			if matches!(
				t,
				Token::KwGoto | Token::KwStop | Token::KwFail | Token::KwWait
			) {
				return false;
			}

			true
		})
		.repeated()
		.at_least(1)
		.at_most(4)
		.collect::<()>()
		.try_map_with_state(|(), span: logos::Span, state: &mut ParseState| {
			if span.len() == 4 {
				Ok(GreenToken::new(
					Syn::StateSprite.into(),
					&state.source[span],
				))
			} else {
				Err(ParseError::custom(
					span,
					"state sprite names must be exactly 4 characters long",
				))
			}
		});

	let hold = primitive::just(Token::StringLit).try_map_with_state(
		|_, span: logos::Span, state: &mut ParseState| {
			if span.len() == 6 {
				Ok(GreenToken::new(
					Syn::StateSprite.into(),
					&state.source[span],
				))
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

/// The returned parser emits a [`Syn::StateFrames`] token.
pub fn state_frames<'i>() -> parser_t!(GreenToken) {
	#[must_use]
	fn is_valid_quoted_char(c: char) -> bool {
		c.is_ascii_alphabetic() || c == '[' || c == ']' || c == '\\' || c == '#'
	}

	let unquoted = primitive::just(Token::Ident).try_map_with_state(
		|_, span: logos::Span, state: &mut ParseState| {
			if !state.source[span.clone()].contains(|c: char| !c.is_ascii_alphabetic()) {
				Ok(GreenToken::new(
					Syn::StateFrames.into(),
					&state.source[span],
				))
			} else {
				Err(ParseError::custom(
					span.clone(),
					format!("invalid frame character string `{}`", &state.source[span]),
				))
			}
		},
	);

	let quoted = primitive::just(Token::StringLit).try_map_with_state(
		|_, span: logos::Span, state: &mut ParseState| {
			let inner = &state.source[(span.start + 1)..(span.end - 1)];

			if !inner.contains(|c: char| !is_valid_quoted_char(c)) {
				Ok(GreenToken::new(
					Syn::StateFrames.into(),
					&state.source[span],
				))
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

pub fn state_duration<'i>() -> parser_t!(GreenElement) {
	primitive::choice((
		int_lit_negative().map(GreenElement::from),
		comb::just_ts(Token::IntLit, Syn::IntLit).map(GreenElement::from),
		expr::expr().map(GreenElement::from),
	))
}

pub fn state_quals<'i>() -> parser_t!(Vec<GreenElement>) {
	let light = primitive::group((
		comb::just_ts(Token::KwLight, Syn::KwLight),
		trivia_0plus(),
		comb::just_ts(Token::ParenL, Syn::ParenL),
		trivia_0plus(),
		primitive::choice((
			comb::just_ts(Token::StringLit, Syn::StringLit),
			comb::just_ts(Token::NameLit, Syn::NameLit),
		)),
		trivia_0plus(),
		comb::just_ts(Token::ParenR, Syn::ParenR),
	))
	.map(|group| coalesce_node(group, Syn::StateLight));

	let offs_num = primitive::choice((
		int_lit_negative(),
		comb::just_ts(Token::IntLit, Syn::IntLit),
	));

	let offset = primitive::group((
		comb::just_ts(Token::KwOffset, Syn::KwOffset),
		trivia_0plus(),
		comb::just_ts(Token::ParenL, Syn::ParenL),
		trivia_0plus(),
		offs_num.clone(),
		trivia_0plus(),
		comb::just_ts(Token::Comma, Syn::Comma),
		trivia_0plus(),
		offs_num,
		trivia_0plus(),
		comb::just_ts(Token::ParenR, Syn::ParenR),
	))
	.map(|group| coalesce_node(group, Syn::StateOffset));

	let qual = primitive::choice((
		comb::just_ts(Token::KwCanRaise, Syn::KwCanRaise).map(GreenElement::from),
		comb::just_ts(Token::KwBright, Syn::KwBright).map(GreenElement::from),
		comb::just_ts(Token::KwSlow, Syn::KwSlow).map(GreenElement::from),
		comb::just_ts(Token::KwNoDelay, Syn::KwNoDelay).map(GreenElement::from),
		comb::just_ts(Token::KwFast, Syn::KwFast).map(GreenElement::from),
		light.map(GreenElement::from),
		offset.map(GreenElement::from),
	));

	primitive::group((trivia_1line(), qual.clone()))
		.or_not()
		.map(|group| match group {
			Some((mut triv, elem)) => {
				triv.push(elem);
				triv
			}
			None => vec![],
		})
		.foldl(
			primitive::group((trivia_1line(), qual)).repeated(),
			|lhs, (mut triv, qual)| {
				let mut elems = lhs;
				elems.append(&mut triv);
				elems.push(qual);
				elems
			},
		)
}

/// The returned parser emits a [`Syn::ActionFunction`] node.
pub fn action_function<'i>() -> parser_t!(GreenNode) {
	let call = primitive::group((
		comb::just_ts(Token::Ident, Syn::Ident),
		primitive::group((
			trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenL),
			trivia_0plus(),
			expr::expr_list(expr::expr()),
			trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR),
		))
		.or_not(),
	))
	.map(|group| coalesce_node(group, Syn::ActionFunction));

	primitive::group((
		trivia_1line(),
		primitive::choice((call,)), // TODO: Anonymous functions.
	))
	.map(|group| coalesce_node(group, Syn::ActionFunction))
}

#[cfg(test)]
mod test {
	use rowan::ast::AstNode;

	use crate::{
		testing::*,
		zdoom::{
			self,
			decorate::{ast, parse::file},
		},
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
	Containment.Area "unruly",0123
	+REFINERY
	DropItem "CMDCENTER" 255 1 PainSound "spawning/vats"

	States(Actor, overlay, ITEM, WeapoN) {
		Spawn: TNT1 A Random(1, 6)
		Wickedly:
			____ "#" 0
			goto super::Spawn.Something + 0
		Repent:
			3HA_ A 1 bright light('perfect')
			"####" B 6 canraise fast nodelay slow A_SpawnItemEx(1, "??")
			"----" "]" -1 offset(-1, 1) light("sever")
			Loop
	}
}
		"#####;

		let tbuf = crate::scan(SOURCE, zdoom::Version::V1_0_0);
		let result = crate::parse(file(), SOURCE, &tbuf);
		let ptree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);

		let cursor = ptree.cursor();
		let toplevel = ast::TopLevel::cast(cursor.first_child().unwrap()).unwrap();

		let actordef = match toplevel {
			ast::TopLevel::ActorDef(inner) => inner,
			other => panic!("expected `ActorDef`, found: {other:#?}"),
		};

		assert_eq!(actordef.name().text(), "hangar");

		assert_eq!(
			actordef
				.base_class()
				.expect("actor definition has no base class")
				.text(),
			"nuclearplant"
		);

		assert_eq!(
			actordef
				.replaced_class()
				.expect("actor definition has no replacement clause")
				.text(),
			"toxinrefinery"
		);

		assert_eq!(
			actordef
				.editor_number()
				.expect("actor definition has no editor number")
				.text()
				.parse::<u16>()
				.expect("actor editor number is not a valid u16"),
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
			other => panic!("expected `StateChange::Goto`, found: {other:#?}"),
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
