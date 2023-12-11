use crate::{
	parser::{OpenMark, Parser},
	zdoom::{
		decorate::{
			parse::{common::*, top::*},
			Syntax,
		},
		Token,
	},
};

pub(super) fn actor_def(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::Ident);
	let mark = p.open();
	p.advance(Syntax::KwActor);
	trivia_0plus(p);
	actor_ident(p);
	trivia_0plus(p);

	if p.at(Token::Colon) {
		let ancestor = p.open();
		p.advance(Syntax::Colon);
		trivia_0plus(p);
		actor_ident(p);
		p.close(ancestor, Syntax::InheritSpec);
		trivia_0plus(p);
	}

	if p.at_str_nc(Token::Ident, "replaces") {
		let replaces = p.open();
		p.advance(Syntax::KwReplaces);
		trivia_0plus(p);
		actor_ident(p);
		p.close(replaces, Syntax::ReplacesClause);
		trivia_0plus(p);
	}

	if p.at(Token::IntLit) {
		let ednum = p.open();
		p.advance(Syntax::IntLit);
		p.close(ednum, Syntax::EditorNumber);
		trivia_0plus(p);
	}

	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		innard(p);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(mark, Syntax::ActorDef);
}

fn innard(p: &mut Parser<Syntax>) {
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
		p.advance_with_error(Syntax::from(token), &[&["an identifier", "`+` or `-`"]]);
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

		p.advance(Syntax::from(token));
	}

	p.close(mark, Syntax::ActorSettings);
}

fn user_var(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwVar);
	let mark = p.open();
	p.advance(Syntax::KwVar);
	trivia_0plus(p);

	if !p.eat_any(&[
		(Token::KwInt, Syntax::KwInt),
		(Token::KwFloat, Syntax::KwFloat),
	]) {
		p.advance_err_and_close(
			mark,
			Syntax::from(p.nth(0)),
			Syntax::Error,
			&[&["`int`", "`float`"]],
		);

		return;
	}

	trivia_0plus(p);
	p.expect(Token::Ident, Syntax::Ident, &[&["an identifier"]]);
	trivia_0plus(p);

	let mut is_array = false;

	if p.at(Token::BracketL) {
		is_array = true;
		let arrlen = p.open();
		p.advance(Syntax::BracketL);
		trivia_0plus(p);
		super::expr::expr(p);
		trivia_0plus(p);
		p.expect(Token::BracketR, Syntax::BracketR, &[&["`]`"]]);
		p.close(arrlen, Syntax::ArrayLen);
	}

	p.expect(
		Token::Semicolon,
		Syntax::Semicolon,
		if is_array {
			&[&["`;`"]]
		} else {
			&[&["`;`", "`[`"]]
		},
	);

	p.close(mark, Syntax::UserVar);
}

pub(super) fn states_block(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwStates);
	let mark = p.open();
	p.advance(Syntax::KwStates);
	trivia_0plus(p);

	if p.at(Token::ParenL) {
		states_usage(p);
	}

	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		let token = p.nth(0);

		// Start with the easy options: non-goto state flow keywords.
		if matches!(
			token,
			Token::KwFail | Token::KwStop | Token::KwLoop | Token::KwWait
		) {
			let flow = p.open();
			p.advance(Syntax::from(token));
			p.close(flow, Syntax::StateFlow);
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
			p.advance(Syntax::Colon);
			p.close(label_or_def, Syntax::StateLabel);
			trivia_0plus(p);
			continue;
		}

		state_def(p, label_or_def);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(mark, Syntax::StatesDef);
}

fn states_usage(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::ParenL);
	let mark = p.open();
	p.advance(Syntax::ParenL);
	trivia_0plus(p);

	fn actor_item_overlay_weapon(p: &mut Parser<Syntax>) {
		p.expect_any_str_nc(
			&[
				(Token::Ident, "actor", Syntax::Ident),
				(Token::Ident, "item", Syntax::Ident),
				(Token::Ident, "overlay", Syntax::Ident),
				(Token::Ident, "weapon", Syntax::Ident),
			],
			&[&["`actor`", "`item`", "`overlay`", "`weapon`"]],
		);
	}

	actor_item_overlay_weapon(p);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		p.expect(Token::Comma, Syntax::Comma, &[&["`,`", "`)`"]]);
		trivia_0plus(p);
		actor_item_overlay_weapon(p);
		trivia_0plus(p);
	}

	p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
	p.close(mark, Syntax::StatesUsage);
	trivia_0plus(p);
}

fn stateflow_goto(p: &mut Parser<Syntax>) {
	let flow = p.open();
	p.advance(Syntax::KwGoto);
	trivia_0plus(p);
	non_whitespace(p);

	if p.find(0, |token| !token.is_trivia()) == Token::Plus {
		trivia_0plus(p);
		p.advance(Syntax::Plus);
		trivia_0plus(p);
		p.expect(Token::IntLit, Syntax::IntLit, &[&["an integer"]]);
	}

	p.close(flow, Syntax::StateFlow);
}

/// Builds a [`Syntax::StateDef`] node.
/// Note that this starts at the trivia after the sprite.
fn state_def(p: &mut Parser<Syntax>, state: OpenMark) {
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

	p.close(state, Syntax::StateDef);
}

fn state_frames(p: &mut Parser<Syntax>) {
	let token = p.nth(0);

	if token == Token::StringLit {
		p.advance(Syntax::NonWhitespace);
		return;
	}

	p.merge(
		Syntax::NonWhitespace,
		|token| {
			is_ident_lax(token)
				|| matches!(
					token,
					Token::BracketL
						| Token::BracketR | Token::Backslash
						| Token::Pound | Token::Pound4
				)
		},
		Syntax::from,
		&[&["a state frame list"]],
	);
}

fn state_duration(p: &mut Parser<Syntax>) {
	let mark = p.open();

	if p.at_str_nc(Token::Ident, "random") {
		p.advance(Syntax::Ident);
		trivia_0plus(p);
		super::expr::arg_list(p);
	} else {
		sign_lit(p);
	}

	p.close(mark, Syntax::StateDuration);
}

fn state_quals(p: &mut Parser<Syntax>) {
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
				p.advance(Syntax::KwLight);
				trivia_0plus(p);
				p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
				trivia_0plus(p);

				p.expect_any(
					&[
						(Token::StringLit, Syntax::StringLit),
						(Token::NameLit, Syntax::NameLit),
					],
					&[&["a string", "a name"]],
				);

				trivia_0plus(p);

				while !p.at(Token::ParenR) && !p.eof() {
					p.expect(Token::Comma, Syntax::Comma, &[&["`,`"]]);
					trivia_0plus(p);
					p.expect_any(
						&[
							(Token::StringLit, Syntax::StringLit),
							(Token::NameLit, Syntax::NameLit),
						],
						&[&["a string", "a name"]],
					);
					trivia_0plus(p);
				}

				trivia_0plus(p);
				p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
				p.close(light, Syntax::StateLight);
			}
			Token::KwOffset => {
				let offset = p.open();
				p.advance(Syntax::KwOffset);
				trivia_0plus(p);
				p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
				trivia_0plus(p);
				sign_lit(p);
				trivia_0plus(p);
				p.expect(Token::Comma, Syntax::Comma, &[&["`,`"]]);
				trivia_0plus(p);
				sign_lit(p);
				trivia_0plus(p);
				p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
				p.close(offset, Syntax::StateOffset);
			}
			t @ (Token::KwBright
			| Token::KwCanRaise
			| Token::KwFast
			| Token::KwSlow
			| Token::KwNoDelay) => p.advance(Syntax::from(t)),
			other => p.advance_with_error(
				Syntax::from(other),
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

	p.close(quals, Syntax::StateQuals);
}

fn action_function(p: &mut Parser<Syntax>) {
	let mark = p.open();

	if p.at(Token::BraceL) {
		compound_statement(p);
	} else {
		p.expect(Token::Ident, Syntax::Ident, &[&["an identifier"]]);

		if p.find(0, |t| !t.is_trivia()) == Token::ParenL {
			trivia_0plus(p);
			super::expr::arg_list(p);
		}
	}

	p.close(mark, Syntax::ActionFunction);
}

pub(super) fn statement(p: &mut Parser<Syntax>) {
	match p.nth(0) {
		Token::KwFor => {
			let mark = p.open();
			p.advance(Syntax::KwFor);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);

			trivia_0plus(p);
			super::expr(p);
			trivia_0plus(p);

			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);

			trivia_0plus(p);
			super::expr(p);
			trivia_0plus(p);

			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);

			trivia_0plus(p);
			super::expr(p);
			trivia_0plus(p);

			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			compound_statement(p);
			p.close(mark, Syntax::ForStat);
		}
		Token::KwWhile => {
			let mark = p.open();
			p.advance(Syntax::KwWhile);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			super::expr::expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			compound_statement(p);
			p.close(mark, Syntax::WhileStat);
		}
		Token::KwDo => {
			let mark = p.open();
			p.advance(Syntax::KwDo);
			trivia_0plus(p);
			compound_statement(p);
			trivia_0plus(p);
			p.expect(Token::KwWhile, Syntax::KwWhile, &[&["`while`"]]);
			trivia_0plus(p);
			p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
			trivia_0plus(p);
			super::expr::expr(p);
			trivia_0plus(p);
			p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(mark, Syntax::DoWhileStat);
		}
		_ => {
			let mark = p.open();
			super::expr::expr(p);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(mark, Syntax::ExprStat);
		}
	}
}

fn compound_statement(p: &mut Parser<Syntax>) {
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		statement(p);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
}

/// Builds a token tagged [`Syntax::NonWhitespace`].
pub(super) fn actor_ident(p: &mut Parser<Syntax>) {
	p.merge(
		Syntax::NonWhitespace,
		|token| !token.is_trivia() && !matches!(token, Token::BraceL | Token::Colon),
		Syntax::from,
		&[&["any non-whitespace"]],
	);
}

/// Gets used for state duration and offset qualifiers.
fn sign_lit(p: &mut Parser<Syntax>) {
	let lit = p.open();

	if p.eat(Token::Minus, Syntax::Minus) {
		p.expect(Token::IntLit, Syntax::IntLit, &[&["an integer"]]);
	} else {
		p.expect_any(
			&[
				(Token::IntLit, Syntax::IntLit),
				(Token::StringLit, Syntax::StringLit),
			],
			&[&["an integer", "a string", "`-`"]],
		);
	}

	p.close(lit, Syntax::SignLit);
	trivia_0plus(p);
}
