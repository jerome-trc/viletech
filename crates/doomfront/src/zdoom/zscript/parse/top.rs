//! Symbolic constants, enums, top-level directives.

use crate::{
	parser::Parser,
	zdoom::{
		zscript::{
			parse::{common::*, expr},
			Syntax,
		},
		Token,
	},
};

/// Builds a [`Syntax::ConstDef`] node.
pub fn const_def(p: &mut Parser<Syntax>) {
	p.debug_assert_at_any(&[Token::KwConst, Token::DocComment]);
	let constdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwConst);
	p.advance(Syntax::KwConst);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::Eq, Syntax::Eq, &[&["`=`"]]);
	trivia_0plus(p);
	expr(p);
	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(constdef, Syntax::ConstDef);
}

/// Builds a [`Syntax::EnumDef`] node.
pub fn enum_def(p: &mut Parser<Syntax>) {
	fn variant(p: &mut Parser<Syntax>) {
		let var = p.open();
		doc_comments(p);
		ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
		trivia_0plus(p);

		if p.eat(Token::Eq, Syntax::Eq) {
			trivia_0plus(p);
			expr(p);
		}

		p.close(var, Syntax::EnumVariant);
	}

	p.debug_assert_at_any(&[Token::KwEnum, Token::DocComment]);
	let enumdef = p.open();
	doc_comments(p);
	p.advance(Syntax::KwEnum);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);

	if p.eat(Token::Colon, Syntax::Colon) {
		trivia_0plus(p);

		p.expect_any(
			&[
				(Token::KwSByte, Syntax::KwSByte),
				(Token::KwByte, Syntax::KwByte),
				(Token::KwInt8, Syntax::KwInt8),
				(Token::KwUInt8, Syntax::KwUInt8),
				(Token::KwShort, Syntax::KwShort),
				(Token::KwUShort, Syntax::KwUShort),
				(Token::KwInt16, Syntax::KwInt16),
				(Token::KwUInt16, Syntax::KwUInt16),
				(Token::KwInt, Syntax::KwInt),
				(Token::KwUInt, Syntax::KwUInt),
			],
			&[&[
				"`sbyte` or `byte` or `int8` or `uint8`",
				"`short` or `ushort` or `int16` or `uint16`",
				"`int` or `uint`",
			]],
		);
	}

	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_no_doc_0plus(p);

	if p.at_if(|token| is_ident_lax(token) || token == Token::DocComment) {
		variant(p);
		trivia_no_doc_0plus(p);

		while !p.at(Token::BraceR) && !p.eof() {
			match p.nth(0) {
				Token::Comma => {
					p.advance(Syntax::Comma);
					trivia_no_doc_0plus(p);
					if p.at_if(|token| is_ident_lax(token) || token == Token::DocComment) {
						doc_comments(p);
						variant(p);
					}
				}
				other => {
					p.advance_with_error(Syntax::from(other), &[&[",", "}"]]);
				}
			}
		}
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);

	if p.find(0, |token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syntax::Semicolon);
	}

	p.close(enumdef, Syntax::EnumDef);
}

/// Builds a [`Syntax::IncludeDirective`] node.
pub fn include_directive(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwInclude);
	let directive = p.open();
	p.expect(Token::KwInclude, Syntax::KwInclude, &[&["`#include`"]]);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syntax::StringLit, &[&["a string"]]);

	while p.find(0, |token| !token.is_trivia()) == Token::StringLit {
		trivia_0plus(p);
		p.advance(Syntax::StringLit);
	}

	p.close(directive, Syntax::IncludeDirective);
}

/// Builds a [`Syntax::VersionDirective`] node.
pub fn version_directive(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwVersion);
	let directive = p.open();
	p.expect(Token::KwVersion, Syntax::KwVersion, &[&["`version`"]]);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syntax::StringLit, &[&["a string"]]);
	p.close(directive, Syntax::VersionDirective);
}
