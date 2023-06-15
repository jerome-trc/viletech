//! Symbolic constants, enums, top-level directives.

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
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

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{zscript::ParseTree, Version},
	};

	use super::*;

	#[test]
	fn smoke_constdef() {
		const SOURCE: &str = r#"const GOLDEN_ANARCHY = BUSHFIRE >>> NONSPECIFIC_TECH_BASE;"#;

		let tbuf = crate::scan(SOURCE, Version::default());
		let parser = ParserBuilder::new(Version::default()).const_def();
		let result = crate::parse(parser, SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
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

		let tbuf = crate::scan(SOURCE, Version::default());
		let parser = ParserBuilder::new(Version::default()).file();
		let result = crate::parse(parser, SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
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

		let tbuf = crate::scan(SOURCE, Version::default());
		let parser = ParserBuilder::new(Version::default()).file();
		let result = crate::parse(parser, SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		assert_no_errors(&ptree);
	}
}
