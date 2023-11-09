use crate::{
	parser::{OpenMark, Parser},
	zdoom::{
		decorate::{
			parse::{common::*, top::*},
			Syn,
		},
		Token,
	},
};

pub(super) fn actor_def(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::Ident);
	let mark = p.open();
	p.advance(Syn::KwActor);
	trivia_0plus(p);
	actor_ident(p);
	trivia_0plus(p);

	if p.at(Token::Colon) {
		let ancestor = p.open();
		p.advance(Syn::Colon);
		trivia_0plus(p);
		actor_ident(p);
		p.close(ancestor, Syn::InheritSpec);
		trivia_0plus(p);
	}

	if p.at_str_nc(Token::Ident, "replaces") {
		let replaces = p.open();
		p.advance(Syn::KwReplaces);
		trivia_0plus(p);
		actor_ident(p);
		p.close(replaces, Syn::ReplacesClause);
		trivia_0plus(p);
	}

	if p.at(Token::IntLit) {
		let ednum = p.open();
		p.advance(Syn::IntLit);
		p.close(ednum, Syn::EditorNumber);
		trivia_0plus(p);
	}

	p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		innard(p);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
	p.close(mark, Syn::ActorDef);
}

fn innard(p: &mut Parser<Syn>) {
	let token = match p.nth(0) {
		Token::KwStates => {
			states_block(p);
			return;
		}
		Token::KwVar => {
			user_var(p);
			return;
		}
		Token::KwConst => {
			const_def(p);
			return;
		}
		Token::KwEnum => {
			enum_def(p);
			return;
		}
		other => other,
	};

	if !is_ident_lax(token) && !matches!(token, Token::Plus | Token::Minus) {
		p.advance_with_error(Syn::from(token), &[&["an identifier", "`+` or `-`"]]);
		return;
	}

	let mark = p.open();

	loop {
		if p.eof() {
			break;
		}

		let token = p.nth(0);

		if token == Token::Eof {
			break;
		}

		if matches!(
			token,
			Token::KwStates | Token::KwVar | Token::KwConst | Token::KwEnum | Token::BraceR
		) {
			break;
		}

		p.advance(Syn::from(token));
	}

	p.close(mark, Syn::ActorSettings);
}

fn user_var(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwVar);
	let mark = p.open();
	p.advance(Syn::KwVar);
	trivia_0plus(p);

	if !p.eat_any(&[(Token::KwInt, Syn::KwInt), (Token::KwFloat, Syn::KwFloat)]) {
		p.advance_err_and_close(
			mark,
			Syn::from(p.nth(0)),
			Syn::Error,
			&[&["`int`", "`float`"]],
		);

		return;
	}

	trivia_0plus(p);
	p.expect(Token::Ident, Syn::Ident, &[&["an identifier"]]);
	trivia_0plus(p);

	let mut is_array = false;

	if p.at(Token::BracketL) {
		is_array = true;
		let arrlen = p.open();
		p.advance(Syn::BracketL);
		trivia_0plus(p);
		super::expr::expr(p);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syn::BracketR, &[&["`]`"]]);
		p.close(arrlen, Syn::ArrayLen);
	}

	p.expect(
		Token::Semicolon,
		Syn::Semicolon,
		if is_array {
			&[&["`;`"]]
		} else {
			&[&["`;`", "`[`"]]
		},
	);

	p.close(mark, Syn::UserVar);
}

pub(super) fn states_block(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwStates);
	let mark = p.open();
	p.advance(Syn::KwStates);
	trivia_0plus(p);

	if p.at(Token::ParenL) {
		states_usage(p);
	}

	p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		let token = p.nth(0);

		// Start with the easy options: non-goto state flow keywords.
		if matches!(
			token,
			Token::KwFail | Token::KwStop | Token::KwLoop | Token::KwWait
		) {
			let flow = p.open();
			p.advance(Syn::from(token));
			p.close(flow, Syn::StateFlow);
			trivia_0plus(p);
			continue;
		}

		if token == Token::KwGoto {
			stateflow_goto(p);
			trivia_0plus(p);
			continue;
		}

		let label_or_def = p.open();
		non_whitespace(p);

		if p.find(0, |token| !token.is_trivia()) == Token::Colon {
			trivia_0plus(p);
			p.advance(Syn::Colon);
			p.close(label_or_def, Syn::StateLabel);
			trivia_0plus(p);
			continue;
		}

		state_def(p, label_or_def);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
	p.close(mark, Syn::StatesDef);
}

fn states_usage(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::ParenL);
	let mark = p.open();
	p.advance(Syn::ParenL);
	trivia_0plus(p);

	fn actor_item_overlay_weapon(p: &mut Parser<Syn>) {
		p.expect_any_str_nc(
			&[
				(Token::Ident, "actor", Syn::Ident),
				(Token::Ident, "item", Syn::Ident),
				(Token::Ident, "overlay", Syn::Ident),
				(Token::Ident, "weapon", Syn::Ident),
			],
			&[&["`actor`", "`item`", "`overlay`", "`weapon`"]],
		);
	}

	actor_item_overlay_weapon(p);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		p.expect(Token::Comma, Syn::Comma, &[&["`,`", "`)`"]]);
		trivia_0plus(p);
		actor_item_overlay_weapon(p);
		trivia_0plus(p);
	}

	p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
	p.close(mark, Syn::StatesUsage);
	trivia_0plus(p);
}

fn stateflow_goto(p: &mut Parser<Syn>) {
	let flow = p.open();
	p.advance(Syn::KwGoto);
	trivia_0plus(p);
	non_whitespace(p);

	if p.find(0, |token| !token.is_trivia()) == Token::Plus {
		trivia_0plus(p);
		p.advance(Syn::Plus);
		trivia_0plus(p);
		p.expect(Token::IntLit, Syn::IntLit, &[&["an integer"]]);
	}

	p.close(flow, Syn::StateFlow);
}

/// Builds a [`Syn::StateDef`] node.
/// Note that this starts at the trivia after the sprite.
fn state_def(p: &mut Parser<Syn>, state: OpenMark) {
	p.assert_at_if(|token| token.is_trivia());

	trivia_0plus(p);
	state_frames(p);
	trivia_0plus(p);
	state_duration(p);
	trivia_0plus(p);
	state_quals(p);
	p.debug_assert_at_if(|t| !t.is_trivia());

	if p.at(Token::BraceL) {
		action_function(p);
	}

	if p.at(Token::Ident) {
		// If the former, this identifier is part of a state label. Nothing to do here.
		//
		// The latter is arguably the worst part of this parser. It is impossible to know
		// whether the next token, if it is an identifier, is a state sprite or a
		// 4 character-long action function name. This will be correct in 99.9% of cases,
		// since no gzdoom.pk3 action functions have names of less than 5 characters.
		// (The shortest are `A_Die`, `A_Log`, and `A_Saw`).
		if !((p.find(1, |t| !t.is_trivia()) == Token::Colon) || (p.nth_span(0).len() == 4)) {
			action_function(p);
		}
	}

	p.close(state, Syn::StateDef);
}

fn state_frames(p: &mut Parser<Syn>) {
	let token = p.nth(0);

	if token == Token::StringLit {
		p.advance(Syn::NonWhitespace);
		return;
	}

	p.merge(
		Syn::NonWhitespace,
		|token| {
			is_ident_lax(token)
				|| matches!(
					token,
					Token::BracketL
						| Token::BracketR | Token::Backslash
						| Token::Pound | Token::Pound4
				)
		},
		Syn::from,
		&[&["a state frame list"]],
	);
}

fn state_duration(p: &mut Parser<Syn>) {
	let mark = p.open();

	if p.at_str_nc(Token::Ident, "random") {
		p.advance(Syn::Ident);
		trivia_0plus(p);
		super::expr::arg_list(p);
	} else {
		sign_lit(p);
	}

	p.close(mark, Syn::StateDuration);
}

fn state_quals(p: &mut Parser<Syn>) {
	let quals = p.open();

	loop {
		if p.eof() {
			break;
		}

		if p.at_any(&[Token::BraceL, Token::Ident]) {
			break;
		}

		if !p.at_if(|token| {
			matches!(
				token,
				Token::KwLight
					| Token::KwOffset | Token::KwBright
					| Token::KwCanRaise | Token::KwFast
					| Token::KwSlow | Token::KwNoDelay
			)
		}) {
			break;
		}

		match p.nth(0) {
			Token::KwLight => {
				let light = p.open();
				p.advance(Syn::KwLight);
				trivia_0plus(p);
				p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);
				trivia_0plus(p);

				p.expect_any(
					&[
						(Token::StringLit, Syn::StringLit),
						(Token::NameLit, Syn::NameLit),
					],
					&[&["a string", "a name"]],
				);

				trivia_0plus(p);

				while !p.at(Token::ParenR) && !p.eof() {
					p.expect(Token::Comma, Syn::Comma, &[&["`,`"]]);
					trivia_0plus(p);
					p.expect_any(
						&[
							(Token::StringLit, Syn::StringLit),
							(Token::NameLit, Syn::NameLit),
						],
						&[&["a string", "a name"]],
					);
					trivia_0plus(p);
				}

				trivia_0plus(p);
				p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
				p.close(light, Syn::StateLight);
			}
			Token::KwOffset => {
				let offset = p.open();
				p.advance(Syn::KwOffset);
				trivia_0plus(p);
				p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);
				trivia_0plus(p);
				sign_lit(p);
				trivia_0plus(p);
				p.expect(Token::Comma, Syn::Comma, &[&["`,`"]]);
				trivia_0plus(p);
				sign_lit(p);
				trivia_0plus(p);
				p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
				p.close(offset, Syn::StateOffset);
			}
			t @ (Token::KwBright
			| Token::KwCanRaise
			| Token::KwFast
			| Token::KwSlow
			| Token::KwNoDelay) => p.advance(Syn::from(t)),
			other => p.advance_with_error(
				Syn::from(other),
				&[&[
					"`bright`",
					"`canraise`",
					"`fast`",
					"`light`",
					"`nodelay`",
					"`offset`",
					"`slow`",
					"`{`",
					"an identifier",
				]],
			),
		}

		trivia_0plus(p);
	}

	p.close(quals, Syn::StateQuals);
}

fn action_function(p: &mut Parser<Syn>) {
	let mark = p.open();

	if p.at(Token::BraceL) {
		compound_statement(p);
	} else {
		p.expect(Token::Ident, Syn::Ident, &[&["an identifier"]]);

		if p.find(0, |t| !t.is_trivia()) == Token::ParenL {
			trivia_0plus(p);
			super::expr::arg_list(p);
		}
	}

	p.close(mark, Syn::ActionFunction);
}

pub(super) fn statement(p: &mut Parser<Syn>) {
	match p.nth(0) {
		Token::KwFor => {
			let mark = p.open();
			p.advance(Syn::KwFor);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);

			trivia_0plus(p);
			super::expr(p);
			trivia_0plus(p);

			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);

			trivia_0plus(p);
			super::expr(p);
			trivia_0plus(p);

			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);

			trivia_0plus(p);
			super::expr(p);
			trivia_0plus(p);

			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			compound_statement(p);
			p.close(mark, Syn::ForStat);
		}
		Token::KwWhile => {
			let mark = p.open();
			p.advance(Syn::KwWhile);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			super::expr::expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			compound_statement(p);
			p.close(mark, Syn::WhileStat);
		}
		Token::KwDo => {
			let mark = p.open();
			p.advance(Syn::KwDo);
			trivia_0plus(p);
			compound_statement(p);
			trivia_0plus(p);
			p.expect(Token::KwWhile, Syn::KwWhile, &[&["`while`"]]);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syn::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			super::expr::expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syn::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(mark, Syn::DoWhileStat);
		}
		_ => {
			let mark = p.open();
			super::expr::expr(p);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
			p.close(mark, Syn::ExprStat);
		}
	}
}

fn compound_statement(p: &mut Parser<Syn>) {
	p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		statement(p);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
}

/// Builds a token tagged [`Syn::NonWhitespace`].
pub(super) fn actor_ident(p: &mut Parser<Syn>) {
	p.merge(
		Syn::NonWhitespace,
		|token| !token.is_trivia() && !matches!(token, Token::BraceL | Token::Colon),
		Syn::from,
		&[&["any non-whitespace"]],
	);
}

/// Gets used for state duration and offset qualifiers.
fn sign_lit(p: &mut Parser<Syn>) {
	let lit = p.open();

	if p.eat(Token::Minus, Syn::Minus) {
		p.expect(Token::IntLit, Syn::IntLit, &[&["an integer"]]);
	} else {
		p.expect_any(
			&[
				(Token::IntLit, Syn::IntLit),
				(Token::StringLit, Syn::StringLit),
			],
			&[&["an integer", "a string", "`-`"]],
		);
	}

	p.close(lit, Syn::SignLit);
	trivia_0plus(p);
}
