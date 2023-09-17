//! Include directives, enum definitions, damage type definitions, symbolic constants.

use crate::{
	parser::Parser,
	zdoom::{decorate::Syn, Token},
};

use super::{common::*, expr::*};

/// Builds a [`Syn::ConstDef`] node.
pub(super) fn const_def(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwConst);
	let mark = p.open();
	p.advance(Syn::KwConst);
	trivia_0plus(p);

	if !p.eat_any(&[(Token::KwFloat, Syn::KwFloat), (Token::KwInt, Syn::KwInt)]) {
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
	p.expect(Token::Eq, Syn::Eq, &[&["`=`"]]);
	trivia_0plus(p);
	expr(p);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);

	p.close(mark, Syn::ConstDef);
}

/// Builds a [`Syn::EnumDef`] node.
pub(super) fn enum_def(p: &mut Parser<Syn>) {
	fn variant(p: &mut Parser<Syn>) {
		let var = p.open();
		ident_lax(p);

		if p.find(0, |t| !t.is_trivia()) == Token::Eq {
			trivia_0plus(p);
			p.advance(Syn::Eq);
			trivia_0plus(p);
			expr(p);
		}

		p.close(var, Syn::EnumVariant);
	}

	p.debug_assert_at(Token::KwEnum);
	let mark = p.open();
	p.advance(Syn::KwEnum);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	let mut trailing_comma = false;

	while !p.at(Token::BraceR) && !p.eof() {
		variant(p);
		trivia_0plus(p);

		if p.eat(Token::Comma, Syn::Comma) {
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
		Syn::BraceR,
		if trailing_comma {
			&[&["`}`"]]
		} else {
			&[&["`}`", "`,`"]]
		},
	);

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
	p.close(mark, Syn::EnumDef);
}

/// Builds a [`Syn::DamageTypeDef`] node.
pub(super) fn damage_type(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::Ident);
	let mark = p.open();
	p.advance(Syn::KwDamageType);
	trivia_0plus(p);
	ident_xlax(p);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &[&["`{`"]]);
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

		p.close(kvp, Syn::DamageTypeKvp);
		trivia_0plus(p);
	}

	p.expect(Token::BraceR, Syn::BraceR, &[&["`}`"]]);
	p.close(mark, Syn::DamageTypeDef);
}

fn sign_lit(p: &mut Parser<Syn>) {
	let lit = p.open();

	if p.eat(Token::Minus, Syn::Minus) {
		p.expect_any(
			&[
				(Token::IntLit, Syn::IntLit),
				(Token::FloatLit, Syn::FloatLit),
			],
			&[&["an integer", "a floating-point number"]],
		);
	} else {
		p.expect_any(
			&[
				(Token::IntLit, Syn::IntLit),
				(Token::FloatLit, Syn::FloatLit),
				(Token::StringLit, Syn::StringLit),
			],
			&[&["an integer", "a floating-point number", "a string", "`-`"]],
		);
	}

	p.close(lit, Syn::SignLit);
	trivia_0plus(p);
}

/// Builds a [`Syn::IncludeDirective`] node.
pub(super) fn include_directive(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwInclude);
	let mark = p.open();
	p.advance(Syn::KwInclude);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syn::StringLit, &[&["a string"]]);
	p.close(mark, Syn::IncludeDirective);
}
