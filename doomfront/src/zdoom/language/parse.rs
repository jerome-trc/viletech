use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::{Gtb12, Gtb8},
	zdoom::lex::Token,
	GreenElement,
};

use super::Syn;

pub fn file<'i>() -> parser_t!(GreenNode) {
	primitive::choice((trivia(), locale_tag(), key_val_pair()))
		.repeated()
		.collect::<Vec<_>>()
		.map(|elems| GreenNode::new(Syn::Root.into(), elems))
		.boxed()
}

pub fn key_val_pair<'i>() -> parser_t!(GreenElement) {
	let ident = primitive::any()
		.filter(|t: &Token| *t == Token::Ident || t.is_keyword())
		.map_with_state(comb::green_token(Syn::Ident));

	let rep = primitive::group((
		trivia_0plus(),
		comb::just_ts(Token::StringLit, Syn::StringLit),
	));

	primitive::group((
		ident,
		trivia_0plus(),
		comb::just_ts(Token::Eq, Syn::Eq),
		rep.repeated().at_least(1).collect::<Vec<_>>(),
		trivia_0plus(),
		primitive::just(Token::Semicolon)
			.map_with_state(comb::green_token(Syn::Semicolon))
			.or_not(),
	))
	.map(|group| {
		let mut gtb = Gtb12::new(Syn::KeyValuePair);
		gtb.push(group.0);
		gtb.append(group.1);
		gtb.push(group.2);

		for (sub_vec, string) in group.3 {
			gtb.append(sub_vec);
			gtb.push(string);
		}

		gtb.append(group.4);
		gtb.maybe(group.5);
		gtb.finish().into()
	})
}

pub fn locale_tag<'i>() -> parser_t!(GreenElement) {
	primitive::group((
		comb::just_ts(Token::BracketL, Syn::BracketL),
		trivia_0plus(),
		comb::just_ts(Token::Ident, Syn::Ident),
		trivia_1plus(),
		comb::just_ts(Token::KwDefault, Syn::KwDefault),
		trivia_0plus(),
		comb::just_ts(Token::BracketR, Syn::BracketR),
	))
	.map(|group| {
		let mut gtb = Gtb8::new(Syn::LocaleTag);
		gtb.push(group.0);
		gtb.append(group.1);
		gtb.push(group.2);
		gtb.append(group.3);
		gtb.push(group.4);
		gtb.append(group.5);
		gtb.push(group.6);
		gtb.finish().into()
	})
}

pub fn trivia<'i>() -> parser_t!(GreenElement) {
	primitive::choice((
		comb::just_ts(Token::Whitespace, Syn::Whitespace),
		comb::just_ts(Token::Comment, Syn::Comment),
	))
	.map(|token| token.into())
}

pub fn trivia_0plus<'i>() -> parser_t!(Vec<GreenElement>) {
	trivia().repeated().collect()
}

pub fn trivia_1plus<'i>() -> parser_t!(Vec<GreenElement>) {
	trivia().repeated().at_least(1).collect()
}

#[cfg(test)]
mod test {
	use std::path::PathBuf;

	use crate::{testing::*, zdoom::language::ParseTree};

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
		let tbuf = crate::scan(source.as_ref());

		let ptree: ParseTree = crate::parse(parser, source.as_ref(), &tbuf);

		assert_no_errors(&ptree);
	}
}
