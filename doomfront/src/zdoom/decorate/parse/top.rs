//! Include directives, enum definitions, damage type definitions, symbolic constants.

use crate::{
	parser::Parser,
	zdoom::{decorate::Syntax, Token},
};

use super::{common::*, expr::*};

/// Builds a [`Syntax::ConstDef`] node.
pub(super) fn const_def(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwConst);
	let mark = p.open();
	p.advance(Syntax::KwConst);
	trivia_0plus(p);

	if !p.eat_any(&[
		(Token::KwFloat, Syntax::KwFloat),
		(Token::KwInt, Syntax::KwInt),
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
	p.expect(Token::Eq, Syntax::Eq, &[&["`=`"]]);
	trivia_0plus(p);
	expr(p);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);

	p.close(mark, Syntax::ConstDef);
}

/// Builds a [`Syntax::EnumDef`] node.
pub(super) fn enum_def(p: &mut Parser<Syntax>) {
	fn variant(p: &mut Parser<Syntax>) {
		let var = p.open();
		ident_lax(p);

		if p.find(0, |t| !t.is_trivia()) == Token::Eq {
			trivia_0plus(p);
			p.advance(Syntax::Eq);
			trivia_0plus(p);
			expr(p);
		}

		p.close(var, Syntax::EnumVariant);
	}

	p.debug_assert_at(Token::KwEnum);
	let mark = p.open();
	p.advance(Syntax::KwEnum);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	let mut trailing_comma = false;

	while !p.at(Token::BraceR) && !p.eof() {
		variant(p);
		trivia_0plus(p);

		if p.eat(Token::Comma, Syntax::Comma) {
			trailing_comma = true;
			trivia_0plus(p);

			if p.at_if(is_ident_lax) {
				trailing_comma = false;
				variant(p);
				trivia_0plus(p);
			}
		}
	}

	p.expect(
		Token::BraceR,
		Syntax::BraceR,
		if trailing_comma {
			&[&["`}`"]]
		} else {
			&[&["`}`", "`,`"]]
		},
	);

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(mark, Syntax::EnumDef);
}

/// Builds a [`Syntax::DamageTypeDef`] node.
pub(super) fn damage_type(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::Ident);
	let mark = p.open();
	p.advance(Syntax::KwDamageType);
	trivia_0plus(p);
	ident_xlax(p);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		let kvp = p.open();
		ident_xlax(p);
		trivia_0plus(p);

		if p.at_any(&[
			Token::IntLit,
			Token::FloatLit,
			Token::StringLit,
			Token::Minus,
		]) {
			sign_lit(p);
		}

		p.close(kvp, Syntax::DamageTypeKvp);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(mark, Syntax::DamageTypeDef);
}

fn sign_lit(p: &mut Parser<Syntax>) {
	let lit = p.open();

	if p.eat(Token::Minus, Syntax::Minus) {
		p.expect_any(
			&[
				(Token::IntLit, Syntax::IntLit),
				(Token::FloatLit, Syntax::FloatLit),
			],
			&[&["an integer", "a floating-point number"]],
		);
	} else {
		p.expect_any(
			&[
				(Token::IntLit, Syntax::IntLit),
				(Token::FloatLit, Syntax::FloatLit),
				(Token::StringLit, Syntax::StringLit),
			],
			&[&["an integer", "a floating-point number", "a string", "`-`"]],
		);
	}

	p.close(lit, Syntax::SignLit);
	trivia_0plus(p);
}

/// Builds a [`Syntax::IncludeDirective`] node.
pub(super) fn include_directive(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwInclude);
	let mark = p.open();
	p.advance(Syntax::KwInclude);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syntax::StringLit, &[&["a string"]]);
	p.close(mark, Syntax::IncludeDirective);
}
