use chumsky::{primitive, recovery, IterParser, Parser};

use crate::{
	comb,
	util::builder::GreenCache,
	zdoom::{
		lex::{Token, TokenStream},
		Extra,
	},
};

use super::syn::Syn;

pub fn file<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let ret = primitive::choice((trivia(), definition()))
		.repeated()
		.collect::<()>();

	#[cfg(any(debug_assertions, test))]
	{
		ret.boxed()
	}
	#[cfg(not(any(debug_assertions, test)))]
	{
		ret
	}
}

pub fn definition<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let ret = comb::node(
		Syn::Definition.into(),
		primitive::group((
			flags(),
			type_spec(),
			trivia_1plus(),
			comb::just(Token::Ident, Syn::Ident.into()),
			default().or_not(),
			trivia_0plus(),
			comb::just(Token::Semicolon, Syn::Semicolon.into()),
		)),
	)
	.recover_with(recovery::via_parser(recovery()));

	#[cfg(any(debug_assertions, test))]
	{
		ret.boxed()
	}
	#[cfg(not(any(debug_assertions, test)))]
	{
		ret
	}
}

pub fn flags<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::Flags.into(),
		primitive::choice((flag(), trivia()))
			.repeated()
			.at_least(1)
			.collect::<()>(),
	)
}

pub fn flag<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		comb::string_nc(Token::Ident, "server", Syn::KwServer.into()),
		comb::string_nc(Token::Ident, "user", Syn::KwUser.into()),
		comb::string_nc(Token::Ident, "nosave", Syn::KwNoSave.into()),
		comb::string_nc(Token::Ident, "noarchive", Syn::KwNoArchive.into()),
		comb::string_nc(Token::Ident, "cheat", Syn::KwCheat.into()),
		comb::string_nc(Token::Ident, "latch", Syn::KwLatch.into()),
	))
	.map(|_| ())
}

pub fn type_spec<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		comb::string_nc(Token::KwInt, "int", Syn::KwInt.into()),
		comb::string_nc(Token::KwFloat, "float", Syn::KwFloat.into()),
		comb::string_nc(Token::KwBool, "bool", Syn::KwBool.into()),
		comb::string_nc(Token::KwColor, "color", Syn::KwColor.into()),
		comb::string_nc(Token::KwString, "string", Syn::KwString.into()),
	))
	.map(|_| ())
}

pub fn default<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::DefaultDef.into(),
		primitive::group((
			trivia_0plus(),
			comb::just(Token::Eq, Syn::Eq.into()),
			trivia_0plus(),
			primitive::choice((
				comb::just(Token::LitFloat, Syn::LitFloat.into()),
				comb::just(Token::LitInt, Syn::LitInt.into()),
				comb::just(Token::KwFalse, Syn::LitFalse.into()),
				comb::just(Token::KwTrue, Syn::LitTrue.into()),
				comb::just(Token::LitString, Syn::LitString.into()),
			)),
		)),
	)
}

pub fn trivia<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		comb::just(Token::Whitespace, Syn::Whitespace.into()),
		comb::just(Token::Comment, Syn::Comment.into()),
	))
	.map(|_| ())
}

fn trivia_0plus<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	trivia().repeated().collect::<()>()
}

fn trivia_1plus<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	trivia().repeated().at_least(1).collect::<()>()
}

fn recovery<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::Error.into(),
		primitive::group((
			primitive::choice((
				trivia(),
				flag(),
				type_spec(),
				comb::just(Token::Eq, Syn::Eq.into()),
				comb::just(Token::LitFloat, Syn::LitFloat.into()),
				comb::just(Token::LitInt, Syn::LitInt.into()),
				comb::just(Token::KwFalse, Syn::LitFalse.into()),
				comb::just(Token::KwTrue, Syn::LitTrue.into()),
				comb::just(Token::LitString, Syn::LitString.into()),
				comb::just(Token::Ident, Syn::Ident.into()),
			))
			.repeated()
			.at_least(1)
			.collect::<()>(),
			primitive::none_of([Token::Semicolon])
				.repeated()
				.collect::<()>(),
			comb::just(Token::Semicolon, Syn::Semicolon.into()),
		)),
	)
}

#[cfg(test)]
mod test {
	use rowan::ast::AstNode;

	use crate::util::{builder::GreenCacheNoop, testing::*};

	use super::*;

	use crate::zdoom::cvarinfo::ast;

	#[test]
	fn smoke() {
		const SOURCE: &str = r#"

// Rue des Acacias
server int egghead_roundabout;
user float acidSurge=0.4	;
cheat noarchive nosave string /* comment? */ BONELESS_VENTURES = "Welcome to the Company !";

	"#;

		let parser = file::<GreenCacheNoop>();

		let ptree = crate::parse(
			parser,
			None,
			Syn::Root.into(),
			SOURCE,
			Token::stream(SOURCE, None),
		);

		assert_no_errors(&ptree);

		let cvars: Vec<_> = ptree
			.cursor::<Syn>()
			.children()
			.map(|c| ast::CVar::cast(c).unwrap())
			.collect();

		assert_eq!(cvars[0].name().text(), "egghead_roundabout");
		assert_eq!(cvars[1].name().text(), "acidSurge");
		assert_eq!(cvars[2].name().text(), "BONELESS_VENTURES");

		assert_eq!(cvars[0].type_spec().kind(), Syn::KwInt);
		assert_eq!(cvars[1].type_spec().kind(), Syn::KwFloat);
		assert_eq!(cvars[2].type_spec().kind(), Syn::KwString);

		let default_0 = cvars[0].default();
		let default_1 = cvars[1].default().unwrap();
		let default_2 = cvars[2].default().unwrap();

		assert_eq!(default_0, None);
		assert_eq!(default_1.literal().kind(), Syn::LitFloat);
		assert_eq!(default_1.literal().text(), "0.4");
		assert_eq!(default_2.literal().kind(), Syn::LitString);
		assert_eq!(default_2.literal().text(), r#""Welcome to the Company !""#);
	}

	#[test]
	fn err_recovery() {
		const SOURCE: &str = r#"

	server int theumpteenthcircle = ;
	user float ICEANDFIRE3 = 0.4;

	"#;

		let parser = file::<GreenCacheNoop>();

		let ptree = crate::parse(
			parser,
			None,
			Syn::Root.into(),
			SOURCE,
			Token::stream(SOURCE, None),
		);

		assert_eq!(ptree.errors.len(), 1);

		let cvar = ast::CVar::cast(ptree.cursor::<Syn>().last_child().unwrap()).unwrap();
		assert_eq!(cvar.name().text(), "ICEANDFIRE3");
	}
}
