use chumsky::{primitive, IterParser, Parser};

use crate::{
	comb,
	util::{builder::GreenCache, state::ParseState},
	zdoom::{
		lex::{Token, TokenStream},
		Extra,
	},
};

use super::Syn;

pub fn file<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((trivia(), locale_tag(), key_val_pair()))
		.repeated()
		.collect::<()>()
		.boxed()
}

pub fn locale_tag<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	comb::node(
		Syn::LocaleTag.into(),
		primitive::group((
			comb::just_ts(Token::BracketL, Syn::BracketL.into()),
			trivia_0plus(),
			comb::just_ts(Token::Ident, Syn::Ident.into()),
			comb::checkpointed(primitive::group((
				trivia_1plus(),
				comb::just_ts(Token::KwDefault, Syn::KwDefault.into()),
			)))
			.or_not(),
			trivia_0plus(),
			comb::just_ts(Token::BracketR, Syn::BracketR.into()),
		)),
	)
}

pub fn key_val_pair<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	let ident = primitive::any()
		.filter(|t: &Token| *t == Token::Ident || t.is_keyword())
		.map_with_state(|_, span, state: &mut ParseState<C>| {
			state.gtb.token(Syn::Ident.into(), &state.source[span]);
		});

	let strings = comb::checkpointed(primitive::group((
		trivia_0plus(),
		comb::just_ts(Token::StringLit, Syn::StringLit.into()),
	)));

	comb::node(
		Syn::KeyValuePair.into(),
		primitive::group((
			ident,
			trivia_0plus(),
			comb::just_ts(Token::Eq, Syn::Eq.into()),
			strings.repeated().at_least(1).collect::<()>(),
			trivia_0plus(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon.into()).or_not(),
		)),
	)
}

pub fn trivia<'i, C>() -> impl 'i + Parser<'i, TokenStream<'i>, (), Extra<'i, C>> + Clone
where
	C: GreenCache,
{
	primitive::choice((
		comb::just_ts(Token::Whitespace, Syn::Whitespace.into()),
		comb::just_ts(Token::Comment, Syn::Comment.into()),
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

#[cfg(test)]
mod test {
	use std::path::PathBuf;

	use crate::{
		util::{builder::GreenCacheNoop, testing::*},
		zdoom::language,
		ParseTree,
	};

	use super::*;

	#[test]
	fn with_sample_data() {
		const ENV_VAR: &str = "DOOMFRONT_LANGUAGE_SAMPLE";

		let path = match std::env::var(ENV_VAR) {
			Ok(v) => PathBuf::from(v),
			Err(_) => {
				eprintln!(
					"Environment variable not set: `{ENV_VAR}`. \
					Cancelling `zdoom::language::parse::test::with_sample_data`."
				);
				return;
			}
		};

		if !path.exists() {
			eprintln!(
				"File does not exist: `{p}`. \
				Cancelling `zdoom::language::parse::test::with_sample_data`.",
				p = path.display(),
			);
			return;
		}

		let bytes = std::fs::read(path)
			.map_err(|err| panic!("File I/O failure: {err}"))
			.unwrap();
		let source = String::from_utf8_lossy(&bytes);

		let parser = file();

		let ptree1 = crate::parse(
			parser,
			Some(GreenCacheNoop),
			Syn::Root.into(),
			source.as_ref(),
			Token::stream(source.as_ref()),
		);

		assert_no_errors(&ptree1);

		let ptree2 = ParseTree::<Token> {
			root: language::parser::file(source.as_ref()).unwrap(),
			errors: vec![],
		};

		assert_no_errors(&ptree2);

		assert_eq!(ptree1.cursor::<Syn>().text(), ptree2.cursor::<Syn>().text());
	}
}
