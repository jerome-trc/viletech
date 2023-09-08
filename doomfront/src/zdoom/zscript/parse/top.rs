//! Symbolic constants, enums, top-level directives.

use crate::{
	parser::Parser,
	zdoom::{
		zscript::{
			parse::{common::*, expr},
			Syn,
		},
		Token,
	},
};

/// Builds a [`Syn::ConstDef`] node.
pub fn const_def(p: &mut Parser<Syn>) {
	p.debug_assert_at_any(&[Token::KwConst, Token::DocComment]);
	let constdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwConst);
	p.advance(Syn::KwConst);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Eq, Syn::Eq, &["`=`"]);
	trivia_0plus(p);
	expr(p);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(constdef, Syn::ConstDef);
}

/// Builds a [`Syn::EnumDef`] node.
pub fn enum_def(p: &mut Parser<Syn>) {
	fn variant(p: &mut Parser<Syn>) {
		let var = p.open();
		ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
		trivia_0plus(p);

		if p.eat(Token::Eq, Syn::Eq) {
			trivia_0plus(p);
			expr(p);
		}

		p.close(var, Syn::EnumVariant);
	}

	p.debug_assert_at_any(&[Token::KwEnum, Token::DocComment]);
	let enumdef = p.open();
	doc_comments(p);
	p.advance(Syn::KwEnum);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);

	if p.eat(Token::Colon, Syn::Colon) {
		trivia_0plus(p);

		p.expect_any(
			&[
				(Token::KwSByte, Syn::KwSByte),
				(Token::KwByte, Syn::KwByte),
				(Token::KwInt8, Syn::KwInt8),
				(Token::KwUInt8, Syn::KwUInt8),
				(Token::KwShort, Syn::KwShort),
				(Token::KwUShort, Syn::KwUShort),
				(Token::KwInt16, Syn::KwInt16),
				(Token::KwUInt16, Syn::KwUInt16),
				(Token::KwInt, Syn::KwInt),
				(Token::KwUInt, Syn::KwUInt),
			],
			&[
				"`sbyte` or `byte` or `int8` or `uint8`",
				"`short` or `ushort` or `int16` or `uint16`",
				"`int` or `uint`",
			],
		);
	}

	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	if p.at_if(|token| is_ident_lax(token) || token == Token::DocComment) {
		doc_comments(p);
		variant(p);
		trivia_0plus(p);

		while !p.at(Token::BraceR) && !p.eof() {
			match p.nth(0) {
				Token::Comma => {
					p.advance(Syn::Comma);
					trivia_0plus(p);
					if p.at_if(|token| is_ident_lax(token) || token == Token::DocComment) {
						doc_comments(p);
						variant(p);
					}
				}
				other => {
					p.advance_with_error(Syn::from(other), &[",", "}"]);
				}
			}
		}
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);

	if p.find(0, |token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syn::Semicolon);
	}

	p.close(enumdef, Syn::EnumDef);
}

/// Builds a [`Syn::IncludeDirective`] node.
pub fn include_directive(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwInclude);
	let directive = p.open();
	p.expect(Token::KwInclude, Syn::KwInclude, &["`#include`"]);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syn::StringLit, &["a string"]);

	while p.find(0, |token| !token.is_trivia()) == Token::StringLit {
		trivia_0plus(p);
		p.advance(Syn::StringLit);
	}

	p.close(directive, Syn::IncludeDirective);
}

/// Builds a [`Syn::VersionDirective`] node.
pub fn version_directive(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwVersion);
	let directive = p.open();
	p.expect(Token::KwVersion, Syn::KwVersion, &["`version`"]);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syn::StringLit, &["a string"]);
	p.close(directive, Syn::VersionDirective);
}
