//! Non-actor top-level elements: symbolic constants, enums, preprocessor directives.

use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb,
	util::builder::GreenCache,
	zdoom::{
		decorate::Syn,
		lex::{Token, TokenStream},
		Extra,
	},
};

use super::{common::*, expr};

pub fn const_def<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let ret = comb::node(
		Syn::ConstDef.into(),
		primitive::group((
			comb::just(Token::KwConst, Syn::KwConst.into()),
			trivia_1plus(),
			primitive::choice((
				comb::string_nc(Token::Ident, "fixed", Syn::KwFixed.into()),
				comb::just(Token::KwFloat, Syn::KwFloat.into()),
				comb::just(Token::KwInt, Syn::KwInt.into()),
			)),
			trivia_1plus(),
			comb::just(Token::Ident, Syn::Ident.into()),
			trivia_0plus(),
			comb::just(Token::Eq, Syn::Eq.into()),
			trivia_0plus(),
			expr::expr(),
			trivia_0plus(),
			comb::just(Token::Semicolon, Syn::Semicolon.into()),
		)),
	);

	#[cfg(any(debug_assertions, test))]
	{
		ret.boxed()
	}
	#[cfg(not(any(debug_assertions, test)))]
	{
		ret
	}
}

pub fn enum_def<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let ret = comb::node(
		Syn::EnumDef.into(),
		primitive::group((
			comb::just(Token::KwEnum, Syn::KwEnum.into()),
			trivia_0plus(),
			comb::just(Token::BraceL, Syn::BraceL.into()),
			trivia_0plus(),
			enum_variants(),
			trivia_0plus(),
			comb::just(Token::BraceR, Syn::BraceR.into()),
			trivia_0plus(),
			comb::just(Token::Semicolon, Syn::Semicolon.into()),
		)),
	);

	#[cfg(any(debug_assertions, test))]
	{
		ret.boxed()
	}
	#[cfg(not(any(debug_assertions, test)))]
	{
		ret
	}
}

fn enum_variants<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let init = comb::node(
		Syn::EnumVariant.into(),
		primitive::group((
			comb::just(Token::Ident, Syn::Ident.into()),
			trivia_0plus(),
			comb::just(Token::Eq, Syn::Eq.into()),
			trivia_0plus(),
			expr::expr(),
		)),
	);

	let uninit = comb::node(
		Syn::EnumVariant.into(),
		comb::just(Token::Ident, Syn::Ident.into()),
	);

	let variant = primitive::choice((init, uninit));

	let successive = comb::checkpointed(primitive::group((
		trivia_0plus(),
		comb::just(Token::Comma, Syn::Comma.into()),
		trivia_0plus(),
		variant.clone(),
	)))
	.repeated()
	.collect::<()>();

	let ret = primitive::group((
		variant,
		successive,
		comb::just(Token::Comma, Syn::Comma.into()).or_not(),
	))
	.map(|_| ());

	#[cfg(any(debug_assertions, test))]
	{
		ret.boxed()
	}
	#[cfg(not(any(debug_assertions, test)))]
	{
		ret
	}
}

pub fn include_directive<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::IncludeDirective.into(),
		primitive::group((
			comb::just(Token::PoundInclude, Syn::PoundInclude.into()),
			trivia_0plus(),
			comb::just(Token::StringLit, Syn::StringLit.into()),
		)),
	)
}

#[cfg(test)]
mod test {
	use rowan::ast::AstNode;

	use crate::{
		util::{builder::GreenCacheNoop, testing::*},
		zdoom::decorate::{ast, SyntaxNode},
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

		let parser = super::enum_def::<GreenCacheNoop>();

		let ptree = crate::parse(
			parser,
			None,
			Syn::Root.into(),
			SOURCE,
			Token::stream(SOURCE, None),
		);
		let cursor = SyntaxNode::new_root(ptree.root);
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

		let parser = super::include_directive::<GreenCacheNoop>();

		let ptree = crate::parse(
			parser,
			None,
			Syn::Root.into(),
			SOURCE,
			Token::stream(SOURCE, None),
		);

		let cursor = SyntaxNode::new_root(ptree.root);

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

		let parser = const_def::<GreenCacheNoop>();

		let ptree = crate::parse(
			parser,
			None,
			Syn::Root.into(),
			SOURCE,
			Token::stream(SOURCE, None),
		);
		let cursor = SyntaxNode::new_root(ptree.root);
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
