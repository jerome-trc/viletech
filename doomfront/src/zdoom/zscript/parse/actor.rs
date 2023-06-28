//! Parsers for parts of definitions for classes inheriting from `Actor`.

use crate::{
	parser::Parser,
	zdoom::{
		zscript::{parse::*, Syn},
		Token,
	},
};

/// Builds a [`Syn::FlagDef`] node.
pub fn flag_def(p: &mut Parser<Syn>) {
	p.debug_assert_at_any(&[Token::KwFlagDef, Token::DocComment]);
	let flagdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwFlagDef);
	p.advance(Syn::KwFlagDef);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Colon, Syn::Colon, &["`:`"]);
	trivia_0plus(p);

	let name = p.nth(0);

	if is_ident_lax(name) {
		p.advance(Syn::Ident);
	} else if name == Token::KwNone {
		p.advance(Syn::KwNone);
	} else {
		p.advance_with_error(Syn::from(p.nth(0)), &["an identifier", "`none`"])
	}

	trivia_0plus(p);
	p.expect(Token::Comma, Syn::Comma, &["`,`"]);
	trivia_0plus(p);
	p.expect(Token::IntLit, Syn::IntLit, &["an integer"]);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(flagdef, Syn::FlagDef);
}

/// Builds a [`Syn::PropertyDef`] node.
pub fn property_def(p: &mut Parser<Syn>) {
	p.debug_assert_at_any(&[Token::KwProperty, Token::DocComment]);
	let propdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwProperty);
	p.advance(Syn::KwProperty);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Colon, Syn::Colon, &["`:`"]);
	trivia_0plus(p);

	let name = p.nth(0);

	if is_ident_lax(name) {
		ident_list::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
	} else if name == Token::KwNone {
		p.advance(Syn::KwNone);
	} else {
		p.advance_with_error(Syn::from(p.nth(0)), &["an identifier", "`none`"])
	}

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(propdef, Syn::PropertyDef);
}

/// Builds a [`Syn::DefaultBlock`] node.
pub fn default_block(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwDefault);
	let defblock = p.open();
	p.advance(Syn::KwDefault);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		let token = p.nth(0);

		if is_ident_lax(token) {
			property_setting(p);
		} else if matches!(token, Token::Plus | Token::Minus) {
			flag_setting(p);
		} else if token == Token::Semicolon {
			p.advance(Syn::Semicolon);
		} else {
			p.advance_with_error(Syn::from(token), &["`+` or `-`", "an identifier", "`;`"]);
		}

		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
	p.close(defblock, Syn::DefaultBlock);
}

/// Builds a [`Syn::FlagSetting`] node.
fn flag_setting(p: &mut Parser<Syn>) {
	p.debug_assert_at_any(&[Token::Plus, Token::Minus]);
	let flag = p.open();

	p.advance(match p.nth(0) {
		Token::Plus => Syn::Plus,
		Token::Minus => Syn::Minus,
		_ => unreachable!(),
	});

	trivia_0plus(p);
	ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);

	if p.find(0, |token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syn::Semicolon);
	}

	p.close(flag, Syn::FlagSetting);
}

/// Builds a [`Syn::PropertySetting`] node.
fn property_setting(p: &mut Parser<Syn>) {
	p.debug_assert_at_if(is_ident_lax);
	let prop = p.open();
	ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
	trivia_0plus(p);

	if !p.at(Token::Semicolon) {
		expr_list(p);
	}

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(prop, Syn::PropertySetting);
}

/// Builds a [`Syn::StatesBlock`] node.
pub fn states_block(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwStates);
	let sblock = p.open();
	p.advance(Syn::KwStates);
	trivia_0plus(p);

	if p.at(Token::ParenL) {
		states_usage(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);

	while !p.at(Token::BraceR) && !p.eof() {
		trivia_0plus(p);

		let token = p.nth(0);

		if is_state_label_part(token) {
			let peeked = p.find(1, |token| !token.is_trivia() && !is_state_label_part(token));

			if matches!(peeked, Token::Colon | Token::Dot) {
				let label = p.open();
				state_label_ident_chain(p);
				trivia_0plus(p);
				p.advance(Syn::Colon);
				p.close(label, Syn::StateLabel);
			} else {
				state_def(p);
			}

			trivia_0plus(p);
			continue;
		}

		match token {
			Token::StringLit => {
				let peeked = p.find(1, |token| !token.is_trivia());

				if matches!(peeked, Token::Colon | Token::Dot) {
					let label = p.open();
					p.advance(Syn::StringLit);
					trivia_0plus(p);
					p.advance(Syn::Colon);
					p.close(label, Syn::StateLabel);
				} else {
					state_def(p);
				}
			}
			Token::IntLit | Token::Minus2 | Token::Pound4 => {
				state_def(p);
			}
			t @ (Token::KwFail | Token::KwStop | Token::KwLoop | Token::KwWait) => {
				let flow = p.open();
				p.advance(Syn::from(t));
				trivia_0plus(p);
				p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
				p.close(flow, Syn::StateFlow);
			}
			Token::KwGoto => {
				let flow = p.open();
				p.advance(Syn::KwGoto);
				trivia_0plus(p);

				if p.eat(Token::KwSuper, Syn::KwSuper) {
					trivia_0plus(p);
					p.expect(Token::Colon2, Syn::Colon2, &["`::`"]);
					trivia_0plus(p);
					ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
				} else if p.at_if(is_ident_lax) {
					let peeked = p.find(0, |token| !token.is_trivia() && !is_ident_lax(token));

					match peeked {
						Token::Dot => ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p),
						Token::Colon2 => {
							p.advance(Syn::Ident);
							trivia_0plus(p);
							p.advance(Syn::Colon2);
							trivia_0plus(p);
							ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
						}
						Token::Semicolon | Token::Plus | Token::Eof => {
							p.advance(Syn::Ident);
						}
						other => p.advance_with_error(
							Syn::from(other),
							&["an identifier", "`.`", "`::`", "`+`", "`;`"],
						),
					}
				} else {
					p.advance_with_error(Syn::from(p.nth(0)), &["an identifier", "`super`"]);
				}

				if p.find(0, |token| !token.is_trivia()) == Token::Plus {
					trivia_0plus(p);
					p.advance(Syn::Plus);
					trivia_0plus(p);
					p.expect(Token::IntLit, Syn::IntLit, &["an integer"]);
				}

				trivia_0plus(p);
				p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
				p.close(flow, Syn::StateFlow);
			}
			other => p.advance_with_error(
				Syn::from(other),
				&[
					"`goto`",
					"`fail`",
					"`loop`",
					"`stop`",
					"`wait`",
					"a state label",
					"a state sprite",
				],
			),
		}

		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
	p.close(sblock, Syn::StatesBlock);
}

#[must_use]
fn is_non_stateflow_keyword(token: Token) -> bool {
	token.is_keyword()
		&& !matches!(
			token,
			Token::KwGoto | Token::KwStop | Token::KwWait | Token::KwLoop | Token::KwFail
		)
}

#[must_use]
fn is_state_label_part(token: Token) -> bool {
	is_ident::<{ ID_SQKW | ID_TYPES }>(token)
		|| is_non_stateflow_keyword(token)
		|| matches!(token, Token::IntLit | Token::NameLit)
}

/// Builds a [`Syn::IdentChain`] node.
fn state_label_ident_chain(p: &mut Parser<Syn>) {
	p.debug_assert_at_if(|token| is_state_label_part(token) || token == Token::Dot);

	let chain = p.open();

	if p.eat(Token::Dot, Syn::Dot) {
		trivia_0plus(p);
	}

	p.merge(
		Syn::Ident,
		is_state_label_part,
		Token::is_trivia,
		Syn::from,
		&["an identifier"],
	);

	while p.find(0, |token| !token.is_trivia()) == Token::Dot {
		trivia_0plus(p);
		p.advance(Syn::Dot);
		trivia_0plus(p);

		p.merge(
			Syn::Ident,
			is_state_label_part,
			Token::is_trivia,
			Syn::from,
			&["an identifier"],
		);
	}

	p.close(chain, Syn::IdentChain);
}

/// Builds a [`Syn::StatesUsage`] node.
pub fn states_usage(p: &mut Parser<Syn>) {
	fn kw(p: &mut Parser<Syn>) {
		p.expect_any_str_nc(
			&[
				(Token::Ident, "actor", Syn::Ident),
				(Token::Ident, "item", Syn::Ident),
				(Token::Ident, "overlay", Syn::Ident),
				(Token::Ident, "weapon", Syn::Ident),
			],
			&["`actor`", "`item`", "`overlay`", "`weapon`"],
		);
	}

	p.debug_assert_at(Token::ParenL);
	let usage = p.open();
	p.advance(Syn::ParenL);
	trivia_0plus(p);
	kw(p);
	trivia_0plus(p);

	while !p.at(Token::ParenR) && !p.eof() {
		if p.eat(Token::Comma, Syn::Comma) {
			trivia_0plus(p);
			kw(p);
			trivia_0plus(p);
		}
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
	p.close(usage, Syn::StatesUsage);
}

/// Builds a [`Syn::StateDef`] node.
pub fn state_def(p: &mut Parser<Syn>) {
	p.debug_assert_at_if(|token| {
		is_ident_lax(token)
			|| token.is_keyword()
			|| matches!(
				token,
				Token::StringLit | Token::IntLit | Token::Minus2 | Token::Pound4
			)
	});

	let state = p.open();
	state_sprite(p);
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
				p.advance(Syn::KwLight);
				trivia_0plus(p);
				p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
				trivia_0plus(p);

				p.expect_any(
					&[
						(Token::StringLit, Syn::StringLit),
						(Token::NameLit, Syn::NameLit),
					],
					&["a string", "a name"],
				);

				trivia_0plus(p);

				while !p.at(Token::ParenR) && !p.eof() {
					p.expect(Token::Comma, Syn::Comma, &["`,`"]);
					trivia_0plus(p);
					p.expect_any(
						&[
							(Token::StringLit, Syn::StringLit),
							(Token::NameLit, Syn::NameLit),
						],
						&["a string", "a name"],
					);
					trivia_0plus(p);
				}

				trivia_0plus(p);
				p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
				p.close(light, Syn::StateLight);
			}
			Token::KwOffset => {
				let offset = p.open();
				p.advance(Syn::KwOffset);
				trivia_0plus(p);
				p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
				trivia_0plus(p);
				expr(p);
				trivia_0plus(p);
				p.expect(Token::Comma, Syn::Comma, &["`,`"]);
				trivia_0plus(p);
				expr(p);
				trivia_0plus(p);
				p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
				p.close(offset, Syn::StateOffset);
			}
			t @ (Token::KwBright
			| Token::KwCanRaise
			| Token::KwFast
			| Token::KwSlow
			| Token::KwNoDelay) => p.advance(Syn::from(t)),
			other => p.advance_with_error(
				Syn::from(other),
				&[
					"`bright`",
					"`canraise`",
					"`fast`",
					"`light`",
					"`nodelay`",
					"`offset`",
					"`slow`",
				],
			),
		}

		trivia_0plus(p);
	}

	p.close(quals, Syn::StateQuals);
	trivia_0plus(p);

	if p.at(Token::Semicolon) {
		p.advance(Syn::Semicolon);
	} else {
		action_function(p);
	}

	p.close(state, Syn::StateDef);
}

fn state_sprite(p: &mut Parser<Syn>) {
	const EXPECTED: &[&str] = &[
		"a state sprite (4 ASCII letters/digits/underscores)",
		"`####`",
		"`----`",
		"\"####\"",
		"\"----\"",
	];

	let token = p.nth(0);

	if is_ident_lax(token) {
		p.advance(Syn::StateSprite);
		return;
	}

	if token.is_keyword() && p.current_slice().len() == 4 {
		p.advance(Syn::StateSprite);
		return;
	}

	match token {
		Token::StringLit => {
			p.advance(Syn::StateSprite);
		}
		Token::IntLit => {
			if p.current_span().len() == 4 {
				p.advance(Syn::StateSprite);
				return;
			}

			p.merge(
				Syn::StateSprite,
				|token| is_ident_lax(token) || token.is_keyword() || token == Token::IntLit,
				Token::is_trivia,
				Syn::from,
				EXPECTED,
			);
		}
		Token::Pound4 => {
			p.advance(Syn::StateSprite);
		}
		Token::Minus2 => {
			if p.nth(1) == Token::Minus2 {
				p.advance_n(Syn::StateSprite, 2);
			} else {
				p.advance_with_error(Syn::from(Token::Minus2), EXPECTED);
			}
		}
		other => {
			p.advance_with_error(Syn::from(other), EXPECTED);
		}
	}
}

fn state_frames(p: &mut Parser<Syn>) {
	let token = p.nth(0);

	if token == Token::StringLit {
		p.advance(Syn::StateFrames);
		return;
	}

	let mut n = 0;

	while !p.at_if(Token::is_trivia) {
		let token = p.nth(n);

		if is_ident_lax(token)
			|| matches!(
				token,
				Token::BracketL | Token::BracketR | Token::Backslash | Token::Pound | Token::Pound4
			) {
			n += 1;
		} else {
			break;
		}
	}

	if n > 0 {
		p.advance_n(Syn::StateFrames, n);
	} else {
		p.advance_with_error(Syn::from(p.nth(0)), &["a state frame list"]);
	}
}

/// Builds a [`Syn::ActionFunction`] node.
fn action_function(p: &mut Parser<Syn>) {
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
		p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	}

	p.close(action, Syn::ActionFunction);
}

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{self, zscript::ParseTree},
	};

	use super::*;

	#[test]
	fn smoke_state_def() {
		const SOURCES: &[&str] = &["#### # 1;", "---- # 1;", "\"####\" \"#\" 1;", "4L4C A 0;"];

		for source in SOURCES {
			let ptree: ParseTree =
				crate::parse(source, state_def, zdoom::lex::Context::ZSCRIPT_LATEST);
			assert_no_errors(&ptree);
			prettyprint_maybe(ptree.cursor());
		}
	}

	#[test]
	fn smoke_goto() {
		const SOURCE: &str = r#####"States {
	goto Super::FrameSetup;
}"#####;

		let ptree: ParseTree =
			crate::parse(SOURCE, states_block, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}
