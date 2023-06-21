//! Symbolic constants, enums, top-level directives.

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{
		zscript::{
			parse::{common::*, expr},
			Syn,
		},
		Token,
	},
};

use super::ParserBuilder;

impl ParserBuilder {
	/// The returned parser emits a [`Syn::ConstDef`] node.
	pub fn const_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwConst, Syn::KwConst),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::Eq, Syn::Eq),
			self.trivia_0plus(),
			self.expr(),
			self.trivia_0plus(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		))
		.map(|group| coalesce_node(group, Syn::ConstDef))
	}

	/// The returned parser emits a [`Syn::EnumDef`] node.
	pub fn enum_def<'i>(&self) -> parser_t!(GreenNode) {
		let variant = primitive::group((
			self.ident(),
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Eq, Syn::Eq),
				self.trivia_0plus(),
				self.expr(),
			))
			.or_not(),
		))
		.map(|group| coalesce_node(group, Syn::EnumVariant));

		let successive = primitive::group((
			self.trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma),
			self.trivia_0plus(),
			variant.clone(),
		));

		primitive::group((
			comb::just_ts(Token::KwEnum, Syn::KwEnum),
			self.trivia_1plus(),
			self.ident(),
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Colon, Syn::Colon),
				self.trivia_0plus(),
				self.ident(),
			))
			.or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.trivia_0plus(),
			primitive::group((variant.or_not(), successive.repeated().collect::<Vec<_>>())),
			self.trivia_0plus(),
			comb::just_ts(Token::Comma, Syn::Comma).or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
			comb::just_ts(Token::Semicolon, Syn::Semicolon).or_not(),
		))
		.map(|group| coalesce_node(group, Syn::EnumDef))
		.boxed()
	}

	/// The returned parser emits a [`Syn::IncludeDirective`] node.
	pub fn include_directive<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::PoundInclude, Syn::PoundInclude),
			self.trivia_0plus(),
			comb::just_ts(Token::StringLit, Syn::StringLit),
		))
		.map(|group| coalesce_node(group, Syn::IncludeDirective))
	}

	/// The returned parser emits a [`Syn::VersionDirective`] node.
	pub fn version_directive<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwVersion, Syn::KwVersion),
			self.trivia_0plus(),
			comb::just_ts(Token::StringLit, Syn::StringLit),
		))
		.map(|group| coalesce_node(group, Syn::VersionDirective))
	}
}

/// Builds a [`Syn::ConstDef`] node.
pub fn const_def(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwConst);
	let constdef = p.open();
	p.expect(Token::KwConst, Syn::KwConst, &["`const`"]);
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
pub fn enum_def(p: &mut crate::parser::Parser<Syn>) {
	fn variant(p: &mut crate::parser::Parser<Syn>) {
		let var = p.open();
		ident_lax(p);
		trivia_0plus(p);

		if p.eat(Token::Eq, Syn::Eq) {
			trivia_0plus(p);
			expr(p);
		}

		p.close(var, Syn::EnumVariant);
	}

	p.debug_assert_at(Token::KwEnum);
	let enumdef = p.open();
	p.expect(Token::KwEnum, Syn::KwEnum, &["`enum`"]);
	trivia_1plus(p);
	ident_lax(p);
	trivia_0plus(p);

	if p.eat(Token::Colon, Syn::Colon) {
		trivia_0plus(p);

		p.expect_any(
			&[
				(Token::KwSByte, Syn::KwSByte),
				(Token::KwByte, Syn::KwByte),
				(Token::KwShort, Syn::KwShort),
				(Token::KwUShort, Syn::KwUShort),
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

	if p.at_if(is_ident_lax) {
		variant(p);
		trivia_0plus(p);

		while !p.at(Token::BraceR) && !p.eof() {
			if p.eat(Token::Comma, Syn::Comma) {
				trivia_0plus(p);
				if p.at_if(is_ident_lax) {
					variant(p);
				}
			}
		}
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);

	if p.next_filtered(|token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syn::Semicolon);
	}

	p.close(enumdef, Syn::EnumDef);
}

/// Builds a [`Syn::IncludeDirective`] node.
pub fn include_directive(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::PoundInclude);
	let directive = p.open();
	p.expect(Token::PoundInclude, Syn::PoundInclude, &["`#include`"]);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syn::StringLit, &["a string"]);
	p.close(directive, Syn::IncludeDirective);
}

/// Builds a [`Syn::VersionDirective`] node.
pub fn version_directive(p: &mut crate::parser::Parser<Syn>) {
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

		let ptree: ParseTree = crate::parse(SOURCE, const_def, zdoom::Version::default());
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

		let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::Version::default());
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

		let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::Version::default());
		assert_no_errors(&ptree);
	}
}
