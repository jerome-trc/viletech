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
	ident(p);
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
		ident_lax(p);
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
			if p.eat(Token::Comma, Syn::Comma) {
				trivia_0plus(p);
				if p.at_if(|token| is_ident_lax(token) || token == Token::DocComment) {
					doc_comments(p);
					variant(p);
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
	p.debug_assert_at(Token::PoundInclude);
	let directive = p.open();
	p.expect(Token::PoundInclude, Syn::PoundInclude, &["`#include`"]);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syn::StringLit, &["a string"]);
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

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{
			self,
			zscript::{parse::file, ParseTree},
		},
	};

	use super::*;

	#[test]
	fn smoke_constdef() {
		const SOURCE: &str = r#"const GOLDEN_ANARCHY = BUSHFIRE >>> NONSPECIFIC_TECH_BASE;"#;

		let ptree: ParseTree = crate::parse(SOURCE, const_def, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_enumdef() {
		const SOURCE: &str = r#"
enum SepticTank {};

enum BeyondTimesGate {
	ELEMENTAL,
}

enum BrickAndRoot {
	CELL_BLOCK_HELL,
	FORGOTTEN_DATA_PROCESSING_CENTER = 1,
	UAC_WELCOME = (9 << 9),
	COLOURS_OF_DOOM = "Ascent",
}
"#;

		let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_directives() {
		const SOURCE: &str = r##"

version "3.7.1"
#include "/summoning/hazard.zs"
#include
"the/pain/maze.zsc"

"##;

		let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
	}
}
