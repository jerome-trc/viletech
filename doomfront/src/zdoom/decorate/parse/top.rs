//! Non-actor top-level elements: symbolic constants, enums, et cetera.

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{decorate::Syn, Token},
	GreenElement,
};

use super::{common::*, expr};

/// The returned parser emits a [`Syn::ConstDef`] node.
pub fn const_def<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		comb::just_ts(Token::KwConst, Syn::KwConst),
		trivia_1plus(),
		primitive::choice((
			comb::string_nc(Token::Ident, "fixed", Syn::KwFixed),
			comb::just_ts(Token::KwFloat, Syn::KwFloat),
			comb::just_ts(Token::KwInt, Syn::KwInt),
		)),
		trivia_1plus(),
		comb::just_ts(Token::Ident, Syn::Ident),
		trivia_0plus(),
		comb::just_ts(Token::Eq, Syn::Eq),
		trivia_0plus(),
		expr::expr(),
		trivia_0plus(),
		comb::just_ts(Token::Semicolon, Syn::Semicolon),
	))
	.map(|group| coalesce_node(group, Syn::ConstDef))
}

/// The returned parser emits a [`Syn::EnumDef`] node.
pub fn enum_def<'i>() -> parser_t!(GreenNode) {
	let ident = primitive::any()
		.filter(|token: &Token| {
			matches!(
				token,
				Token::Ident
					| Token::KwBright | Token::KwFast
					| Token::KwSlow | Token::KwNoDelay
					| Token::KwCanRaise | Token::KwOffset
					| Token::KwLight
			)
		})
		.map_with_state(comb::green_token(Syn::Ident));

	let variant = primitive::group((
		ident,
		primitive::group((
			trivia_0plus(),
			comb::just_ts(Token::Eq, Syn::Eq),
			trivia_0plus(),
			expr(),
		))
		.or_not(),
	))
	.map(|group| coalesce_node(group, Syn::EnumVariant));

	let successive = primitive::group((
		trivia_0plus(),
		comb::just_ts(Token::Comma, Syn::Comma),
		trivia_0plus(),
		variant.clone(),
	));

	primitive::group((
		comb::just_ts(Token::KwEnum, Syn::KwEnum),
		trivia_0plus(),
		comb::just_ts(Token::BraceL, Syn::BraceL),
		trivia_0plus(),
		primitive::group((variant.or_not(), successive.repeated().collect::<Vec<_>>())),
		trivia_0plus(),
		comb::just_ts(Token::Comma, Syn::Comma).or_not(),
		trivia_0plus(),
		comb::just_ts(Token::BraceR, Syn::BraceR),
		trivia_0plus(),
		comb::just_ts(Token::Semicolon, Syn::Semicolon),
	))
	.map(|group| coalesce_node(group, Syn::EnumDef))
}

/// The returned parser emits a [`Syn::DamageTypeDef`] node.
pub fn damage_type_def<'i>() -> parser_t!(GreenNode) {
	let ident = primitive::any()
		.filter(|t: &Token| *t == Token::Ident || t.is_keyword())
		.map_with_state(comb::green_token(Syn::Ident));

	let kvp = primitive::group((
		ident.clone(),
		(primitive::group((
			trivia_1plus(),
			primitive::choice((
				int_lit_negative(),
				float_lit_negative(),
				comb::just_ts(Token::IntLit, Syn::IntLit),
				comb::just_ts(Token::FloatLit, Syn::FloatLit),
				comb::just_ts(Token::StringLit, Syn::StringLit),
			)),
		)))
		.or_not(),
	))
	.map(|group| coalesce_node(group, Syn::DamageTypeKvp));

	primitive::group((
		comb::string_nc(Token::Ident, "damagetype", Syn::KwDamageType),
		trivia_1plus(),
		ident,
		trivia_0plus(),
		comb::just_ts(Token::BraceL, Syn::BraceL),
		primitive::choice((trivia(), kvp.clone().map(GreenElement::from)))
			.repeated()
			.collect::<Vec<_>>(),
		comb::just_ts(Token::BraceR, Syn::BraceR),
	))
	.map(|group| coalesce_node(group, Syn::DamageTypeDef))
}

/// The returned parser emits a [`Syn::IncludeDirective`] node.
pub fn include_directive<'i>() -> parser_t!(GreenNode) {
	primitive::group((
		comb::just_ts(Token::PoundInclude, Syn::PoundInclude),
		trivia_0plus(),
		comb::just_ts(Token::StringLit, Syn::StringLit),
	))
	.map(|group| coalesce_node(group, Syn::IncludeDirective))
}

#[cfg(test)]
mod test {
	use rowan::ast::AstNode;

	use crate::{
		testing::*,
		zdoom::{
			self,
			decorate::{ast, parse::file, ParseTree},
		},
	};

	use super::*;

	#[test]
	fn enum_def() {
		const SOURCE: &str = r#"

enum {
	LIMBO,
	DIS = LIMBO,
	WARRENS = 0,
	MYST_FORT = 9.9,
	MT_EREBUS = false,
	CATHEDRAL = "Yes, string literals are valid enum initializers in DECORATE!",
}; // Floats and booleans too.

"#;

		let tbuf = crate::scan(SOURCE, zdoom::Version::V1_0_0);
		let result = crate::parse(file(), SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);

		let cursor = ptree.cursor();
		let enumdef = ast::TopLevel::cast(cursor.children().next().unwrap())
			.unwrap()
			.into_enumdef()
			.unwrap();
		let mut variants = enumdef.variants();

		let var1 = variants.next().unwrap();
		assert_eq!(var1.name().text(), "LIMBO");
		assert!(var1.initializer().is_none());

		let var2 = variants.next().unwrap();
		assert_eq!(var2.name().text(), "DIS");
		assert_eq!(
			var2.initializer()
				.unwrap()
				.into_ident()
				.unwrap()
				.token()
				.text(),
			"LIMBO"
		);

		let var7 = variants.last().unwrap();

		assert_eq!(var7.name().text(), "CATHEDRAL");
		assert_eq!(
			var7.initializer()
				.unwrap()
				.into_literal()
				.unwrap()
				.token()
				.string()
				.unwrap(),
			"Yes, string literals are valid enum initializers in DECORATE!"
		);
	}

	#[test]
	fn include_directive() {
		const SOURCE: &str =
			" #InClUdE \"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\"";

		let tbuf = crate::scan(SOURCE, zdoom::Version::V1_0_0);
		let result = crate::parse(file(), SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);

		let cursor = ptree.cursor();

		assert_sequence(
			&[
				(Syn::Root, None),
				(Syn::Whitespace, Some(" ")),
				(Syn::IncludeDirective, None),
				(Syn::PoundInclude, Some("#InClUdE")),
				(Syn::Whitespace, Some(" ")),
				(
					Syn::StringLit,
					Some("\"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\""),
				),
			],
			cursor.clone(),
		);

		let incdirect = match ast::TopLevel::cast(cursor.first_child().unwrap()).unwrap() {
			ast::TopLevel::IncludeDirective(inner) => inner,
			other => panic!("Expected `IncludeDirective`, found: {other:#?}"),
		};

		assert_eq!(
			incdirect.path().text(),
			"\"actors/misc/DevelopersDevelopersDevelopersDevelopers.txt\""
		);
	}

	#[test]
	fn symbolic_constants() {
		const SOURCE: &str = r##"

const /* bools */ int KNEE_DEEP = 1234567890;
const fixed /* are */ SHORES = 9.0000000;
const float INFERNO /* forbidden */ = 0.9999999;

"##;

		let tbuf = crate::scan(SOURCE, zdoom::Version::V1_0_0);
		let result = crate::parse(file(), SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);

		let cursor = ptree.cursor();

		let mut constdefs = cursor
			.children()
			.map(|node| ast::TopLevel::cast(node).unwrap().into_constdef().unwrap());

		let constdef1 = constdefs.next().unwrap();

		assert_eq!(constdef1.name().text(), "KNEE_DEEP");
		assert_eq!(constdef1.type_spec(), ast::ConstType::Int);
		assert_eq!(
			constdef1
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.int()
				.unwrap()
				.unwrap(),
			1234567890
		);

		let constdef2 = constdefs.next().unwrap();

		assert_eq!(constdef2.name().text(), "SHORES");
		assert_eq!(constdef2.type_spec(), ast::ConstType::Fixed);
		assert_eq!(
			constdef2
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap(),
			9.0000000
		);

		let constdef3 = constdefs.next().unwrap();

		assert_eq!(constdef3.name().text(), "INFERNO");
		assert_eq!(constdef3.type_spec(), ast::ConstType::Float);
		assert_eq!(
			constdef3
				.expr()
				.into_literal()
				.unwrap()
				.token()
				.float()
				.unwrap(),
			0.9999999
		);
	}
}
