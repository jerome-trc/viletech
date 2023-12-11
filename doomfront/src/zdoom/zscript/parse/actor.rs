//! Parsers for parts of definitions for classes inheriting from `Actor`.

use crate::{
	parser::{OpenMark, Parser},
	zdoom::{
		zscript::{parse::*, Syntax},
		Token,
	},
};

/// Builds a [`Syntax::FlagDef`] node.
pub fn flag_def(p: &mut Parser<Syntax>) {
	p.debug_assert_at_any(&[Token::KwFlagDef, Token::DocComment]);
	let flagdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwFlagDef);
	p.advance(Syntax::KwFlagDef);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Colon, Syntax::Colon, &[&["`:`"]]);
	trivia_0plus(p);

	let name = p.nth(0);

	if is_ident_lax(name) {
		p.advance(Syntax::Ident);
	} else if name == Token::KwNone {
		p.advance(Syntax::KwNone);
	} else {
		p.advance_with_error(Syntax::from(p.nth(0)), &[&["an identifier", "`none`"]])
	}

	trivia_0plus(p);
	p.expect(Token::Comma, Syntax::Comma, &[&["`,`"]]);
	trivia_0plus(p);
	p.expect(Token::IntLit, Syntax::IntLit, &[&["an integer"]]);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(flagdef, Syntax::FlagDef);
}

/// Builds a [`Syntax::PropertyDef`] node.
pub fn property_def(p: &mut Parser<Syntax>) {
	p.debug_assert_at_any(&[Token::KwProperty, Token::DocComment]);
	let propdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwProperty);
	p.advance(Syntax::KwProperty);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Colon, Syntax::Colon, &[&["`:`"]]);
	trivia_0plus(p);

	let name = p.nth(0);

	if is_ident_lax(name) {
		ident_list::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
	} else if name == Token::KwNone {
		p.advance(Syntax::KwNone);
	} else {
		p.advance_with_error(Syntax::from(p.nth(0)), &[&["an identifier", "`none`"]])
	}

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(propdef, Syntax::PropertyDef);
}

/// Builds a [`Syntax::DefaultBlock`] node.
pub fn default_block(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwDefault);
	let defblock = p.open();
	p.advance(Syntax::KwDefault);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		let token = p.nth(0);

		if is_ident_lax(token) {
			property_setting(p);
		} else if matches!(token, Token::Plus | Token::Minus) {
			flag_setting(p);
		} else if token == Token::Semicolon {
			p.advance(Syntax::Semicolon);
		} else {
			p.advance_with_error(
				Syntax::from(token),
				&[&["`+` or `-`", "an identifier", "`;`"]],
			);
		}

		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(defblock, Syntax::DefaultBlock);
}

/// Builds a [`Syntax::FlagSetting`] node.
fn flag_setting(p: &mut Parser<Syntax>) {
	p.debug_assert_at_any(&[Token::Plus, Token::Minus]);
	let flag = p.open();

	p.advance(match p.nth(0) {
		Token::Plus => Syntax::Plus,
		Token::Minus => Syntax::Minus,
		_ => unreachable!(),
	});

	trivia_0plus(p);
	ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);

	if p.find(0, |token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syntax::Semicolon);
	}

	p.close(flag, Syntax::FlagSetting);
}

/// Builds a [`Syntax::PropertySetting`] node.
fn property_setting(p: &mut Parser<Syntax>) {
	p.debug_assert_at_if(is_ident_lax);
	let prop = p.open();
	ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
	trivia_0plus(p);

	if !p.at(Token::Semicolon) {
		expr_list(p);
	}

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(prop, Syntax::PropertySetting);
}

/// Builds a [`Syntax::StatesBlock`] node.
pub fn states_block(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwStates);
	let sblock = p.open();
	p.advance(Syntax::KwStates);
	trivia_0plus(p);

	if p.at(Token::ParenL) {
		states_usage(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		trivia_0plus(p);

		let token = p.nth(0);

		// Start with the easy options: non-goto state flow keywords.
		if matches!(
			token,
			Token::KwFail | Token::KwStop | Token::KwLoop | Token::KwWait
		) {
			stateflow_simple(p);
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
			trivia_0plus(p);
			p.close(label_or_def, Syntax::StateLabel);
			continue;
		}

		state_def(p, label_or_def);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(sblock, Syntax::StatesBlock);
}

fn stateflow_simple(p: &mut Parser<Syntax>) {
	let flow = p.open();
	p.advance(Syntax::from(p.nth(0)));
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(flow, Syntax::StateFlow);
}

fn stateflow_goto(p: &mut Parser<Syntax>) {
	let flow = p.open();
	p.advance(Syntax::KwGoto);
	trivia_0plus(p);

	if p.eat(Token::KwSuper, Syntax::KwSuper) {
		trivia_0plus(p);
		p.expect(Token::Colon2, Syntax::Colon2, &[&["`::`"]]);
		trivia_0plus(p);
		ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
	} else if p.at_if(is_ident_lax) {
		let peeked = p.find(0, |token| !token.is_trivia() && !is_ident_lax(token));

		match peeked {
			Token::Dot => ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p),
			Token::Colon2 => {
				p.advance(Syntax::Ident);
				trivia_0plus(p);
				p.advance(Syntax::Colon2);
				trivia_0plus(p);
				ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
			}
			Token::Semicolon | Token::Plus | Token::Eof => {
				ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
			}
			other => p.advance_with_error(
				Syntax::from(other),
				&[&["an identifier", "`::`", "`+`", "`;`"]],
			),
		}
	} else {
		p.advance_with_error(Syntax::from(p.nth(0)), &[&["an identifier", "`super`"]]);
	}

	if p.find(0, |token| !token.is_trivia()) == Token::Plus {
		trivia_0plus(p);
		p.advance(Syntax::Plus);
		trivia_0plus(p);
		p.expect(Token::IntLit, Syntax::IntLit, &[&["an integer"]]);
	}

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(flow, Syntax::StateFlow);
}

fn non_whitespace(p: &mut Parser<Syntax>) {
	p.merge(
		Syntax::NonWhitespace,
		|token| !token.is_trivia() && token != Token::Colon,
		Syntax::from,
		&[&["any non-whitespace"]],
	);
}

/// Builds a [`Syntax::StatesUsage`] node.
pub(super) fn states_usage(p: &mut Parser<Syntax>) {
	fn kw(p: &mut Parser<Syntax>) {
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

	p.debug_assert_at(Token::ParenL);
	let usage = p.open();
	p.advance(Syntax::ParenL);
	trivia_0plus(p);
	kw(p);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		if p.eat(Token::Comma, Syntax::Comma) {
			trivia_0plus(p);
			kw(p);
			trivia_0plus(p);
		}
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
	p.close(usage, Syntax::StatesUsage);
}

/// Builds a [`Syntax::StateDef`] node.
/// Note that this starts at the trivia after the sprite.
fn state_def(p: &mut Parser<Syntax>, state: OpenMark) {
	p.assert_at_if(|token| token.is_trivia());

	trivia_0plus(p);
	state_frames(p);
	trivia_0plus(p);
	expr(p);
	trivia_0plus(p);

	let quals = p.open();

	loop {
		if p.eof() {
			break;
		}

		if p.at_any(&[Token::Semicolon, Token::BraceL, Token::Ident]) {
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
				expr(p);
				trivia_0plus(p);
				p.expect(Token::Comma, Syntax::Comma, &[&["`,`"]]);
				trivia_0plus(p);
				expr(p);
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
				]],
			),
		}

		trivia_0plus(p);
	}

	p.close(quals, Syntax::StateQuals);
	trivia_0plus(p);

	if p.at(Token::Semicolon) {
		p.advance(Syntax::Semicolon);
	} else {
		action_function(p);
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
		|t| {
			is_ident_lax(t)
				|| matches!(
					t,
					Token::BracketL
						| Token::BracketR | Token::Backslash
						| Token::Pound | Token::Pound4
				)
		},
		Syntax::from,
		&[&["a state frame list"]],
	);
}

/// Builds a [`Syntax::ActionFunction`] node.
fn action_function(p: &mut Parser<Syntax>) {
	let action = p.open();

	if p.at(Token::BraceL) {
		compound_stat(p);
	} else {
		ident_lax(p);
		trivia_0plus(p);

		if p.at(Token::ParenL) {
			expr::arg_list(p);
		}

		trivia_0plus(p);
		p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	}

	p.close(action, Syntax::ActionFunction);
}
