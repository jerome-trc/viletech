use chumsky::{primitive, recovery, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{comb, parser_t, parsing::*, zdoom::lex::Token, GreenElement, ParseState};

use super::Syn;

/// The returned parser emits a [`Syn::Root`] node.
pub fn file<'i>() -> parser_t!(GreenNode) {
	primitive::choice((trivia(), header(), key_val_pair()))
		.repeated()
		.collect::<Vec<_>>()
		.map(|elems| GreenNode::new(Syn::Root.into(), elems))
		.boxed()
}

/// The returned parser emits a [`Syn::KeyValuePair`] node.
pub fn key_val_pair<'i>() -> parser_t!(GreenElement) {
	let ident = primitive::any()
		.filter(|t: &Token| *t == Token::Ident || t.is_keyword())
		.map_with_state(comb::green_token(Syn::Ident));

	let rep = primitive::group((
		trivia_0plus(),
		comb::just_ts(Token::StringLit, Syn::StringLit),
	));

	let ifgame = primitive::group((
		comb::just_ts(Token::Dollar, Syn::Dollar),
		trivia_0plus(),
		comb::string_nc(Token::Ident, "ifgame", Syn::KwIfGame),
		trivia_0plus(),
		comb::just_ts(Token::ParenL, Syn::ParenL),
		trivia_0plus(),
		ident.clone(),
		trivia_0plus(),
		comb::just_ts(Token::ParenR, Syn::ParenR),
		trivia_0plus(),
	))
	.map(|group| coalesce_node(group, Syn::GameQualifier));

	primitive::group((
		ifgame.or_not(),
		ident,
		trivia_0plus(),
		comb::just_ts(Token::Eq, Syn::Eq),
		rep.repeated().at_least(1).collect::<Vec<_>>(),
		trivia_0plus(),
		comb::just_ts(Token::Semicolon, Syn::Semicolon).or_not(),
	))
	.map(|group| coalesce_node(group, Syn::KeyValuePair).into())
	.recover_with(recovery::via_parser(recover_token(
		[Token::Dollar, Token::Ident],
		[Token::Ident, Token::BracketL],
	)))
}

/// The returned parser emits a [`Syn::Header`] node.
pub fn header<'i>() -> parser_t!(GreenElement) {
	let content = primitive::choice((
		comb::just_ts(Token::Tilde, Syn::Tilde),
		comb::just_ts(Token::KwDefault, Syn::KwDefault),
		comb::just_ts(Token::Asterisk, Syn::Asterisk),
		primitive::any()
			.filter(|token: &Token| token.is_keyword() || *token == Token::Ident)
			.map_with_state(comb::green_token(Syn::Ident)),
	));

	let rep = primitive::group((trivia_1plus(), content.clone()));

	primitive::group((
		comb::just_ts(Token::BracketL, Syn::BracketL),
		trivia_0plus(),
		content,
		rep.repeated().collect::<Vec<_>>(),
		trivia_0plus(),
		comb::just_ts(Token::BracketR, Syn::BracketR),
	))
	.map(|group| coalesce_node(group, Syn::Header).into())
}

fn recover_token<'i, S, U>(start: S, until: U) -> parser_t!(GreenElement)
where
	S: 'i + chumsky::container::Seq<'i, Token> + Copy,
	U: 'i + chumsky::container::Seq<'i, Token> + Copy,
{
	let mapper = |token, span, state: &mut ParseState| {
		let syn = match token {
			Token::Whitespace => Syn::Whitespace,
			Token::Comment => Syn::Comment,
			_ => Syn::Unknown,
		};

		GreenToken::new(syn.into(), &state.source[span])
	};

	primitive::group((
		primitive::one_of(start).map_with_state(mapper),
		primitive::none_of(until)
			.map_with_state(mapper)
			.repeated()
			.at_least(1)
			.collect::<Vec<_>>(),
	))
	.map(|(start, mut following)| {
		following.insert(0, start);

		GreenNode::new(
			Syn::Error.into(),
			following.into_iter().map(|token| token.into()),
		)
		.into()
	})
}

/// The returned parser emits a [`Syn::Whitespace`] or [`Syn::Comment`] token.
fn trivia<'i>() -> parser_t!(GreenElement) {
	primitive::choice((
		comb::just_ts(Token::Whitespace, Syn::Whitespace),
		comb::just_ts(Token::Comment, Syn::Comment),
		comb::just_ts(Token::RegionStart, Syn::RegionStart),
		comb::just_ts(Token::RegionEnd, Syn::RegionEnd),
	))
	.map(|token| token.into())
}

/// Shorthand for `self.trivia().repeated().collect()`.
fn trivia_0plus<'i>() -> parser_t!(Vec<GreenElement>) {
	trivia().repeated().collect()
}

/// Shorthand for `self.trivia().repeated().at_least(1).collect()`.
fn trivia_1plus<'i>() -> parser_t!(Vec<GreenElement>) {
	trivia().repeated().at_least(1).collect()
}

pub mod hand {
	use super::*;

	impl crate::parser::LangExt for Syn {
		type Token = Token;
		const EOF: Self::Token = Token::Eof;
		const ERR_NODE: Self::Kind = Syn::Error;
	}

	pub fn _file(p: &mut crate::parser::Parser<Syn>) {
		let root = p.open();

		while !p.eof() {
			if p.at_if(|t| matches!(t, Token::Ident | Token::Dollar) || t.is_keyword()) {
				_kvp(p);
			} else if p.at(Token::BracketL) {
				_header(p);
			} else if _trivia(p) {
			} else {
				p.advance_with_error(Syn::Unknown);
			}
		}

		p.close(root, Syn::Root);
	}

	pub fn _kvp(p: &mut crate::parser::Parser<Syn>) {
		debug_assert!(p.at_if(|t| matches!(t, Token::Ident | Token::Dollar)));

		let kvp = p.open();

		if p.at(Token::Dollar) {
			_ifgame(p);
		}

		p.expect_if(|t| t == Token::Ident || t.is_keyword(), Syn::Ident);
		_trivia_0plus(p);
		p.expect(Token::Eq, Syn::Eq);

		loop {
			if p.at_any(&[Token::Eof, Token::Semicolon, Token::Ident, Token::BracketL]) {
				break;
			}

			_trivia_0plus(p);

			if p.at_any(&[Token::Eof, Token::Semicolon, Token::Ident, Token::BracketL]) {
				break;
			}

			p.expect(Token::StringLit, Syn::StringLit);
		}

		_trivia_0plus(p);
		p.eat(Token::Semicolon, Syn::Semicolon);

		p.close(kvp, Syn::KeyValuePair);
	}

	fn _ifgame(p: &mut crate::parser::Parser<Syn>) {
		debug_assert!(p.at(Token::Dollar));

		let ifgame = p.open();

		p.expect(Token::Dollar, Syn::Dollar);
		_trivia_0plus(p);
		p.expect_str_nc(Token::Ident, "ifgame", Syn::KwIfGame);
		_trivia_0plus(p);
		p.expect(Token::ParenL, Syn::ParenL);
		_trivia_0plus(p);
		p.expect_if(|t| t == Token::Ident || t.is_keyword(), Syn::Ident);
		_trivia_0plus(p);
		p.expect(Token::ParenR, Syn::ParenR);
		_trivia_0plus(p);

		p.close(ifgame, Syn::GameQualifier);
	}

	pub fn _header(p: &mut crate::parser::Parser<Syn>) {
		debug_assert!(p.at(Token::BracketL));

		let header = p.open();

		p.expect(Token::BracketL, Syn::BracketL);

		while !p.at(Token::BracketR) && !p.eof() {
			if _trivia(p) {
				continue;
			}

			let token = p.nth(0);

			if token == Token::Ident || token.is_keyword() {
				p.advance(Syn::Ident);
				continue;
			}

			match token {
				Token::Tilde => p.advance(Syn::Tilde),
				Token::KwDefault => p.advance(Syn::KwDefault),
				Token::Asterisk => p.advance(Syn::Asterisk),
				_ => {
					if !p.eof() {
						p.advance(Syn::Unknown);
					}

					p.close(header, Syn::Error);
					return;
				}
			}
		}

		p.expect(Token::BracketR, Syn::BracketR);

		p.close(header, Syn::Header);
	}

	fn _trivia(p: &mut crate::parser::Parser<Syn>) -> bool {
		p.eat(Token::Whitespace, Syn::Whitespace) || p.eat(Token::Comment, Syn::Comment)
	}

	fn _trivia_0plus(p: &mut crate::parser::Parser<Syn>) {
		loop {
			if !_trivia(p) {
				return;
			}
		}
	}

	fn _trivia_1plus(p: &mut crate::parser::Parser<Syn>) {
		p.expect_any(&[
			(Token::Whitespace, Syn::Whitespace),
			(Token::Comment, Syn::Comment),
		]);

		loop {
			if !_trivia(p) {
				return;
			}
		}
	}
}

#[cfg(test)]
mod test {
	use std::path::PathBuf;

	use rowan::ast::AstNode;

	use crate::{
		testing::*,
		zdoom::{
			language::{ast, ParseTree},
			Version,
		},
	};

	use super::*;

	#[test]
	fn smoke() {
		const SOURCE: &str = r#"
[enu * ~ default]

$ifgame(harmony) THE_UNDERWATER_LAB = "Echidna";
MEGALOPOLIS = "The Omega";
"#;

		let tbuf = crate::scan(SOURCE, Version::default());
		let result = crate::parse(file(), SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		let mut ast = ptree.cursor().children();

		let header = ast::Header::cast(ast.next().unwrap()).unwrap();
		let mut header_contents = header.contents();
		assert_eq!(header_contents.next().unwrap().text(), "enu");
		assert_eq!(header_contents.next().unwrap().text(), "*");
		assert_eq!(header_contents.next().unwrap().text(), "~");
		assert_eq!(header_contents.next().unwrap().text(), "default");

		let kvp0 = ast::KeyValuePair::cast(ast.next().unwrap()).unwrap();
		assert_eq!(kvp0.game_qualifier().unwrap().game_id().text(), "harmony");
		assert_eq!(kvp0.key().text(), "THE_UNDERWATER_LAB");
		assert_eq!(
			&kvp0
				.string_parts()
				.map(|token| token.text().to_string())
				.collect::<String>(),
			r#""Echidna""#
		);

		let kvp1 = ast::KeyValuePair::cast(ast.next().unwrap()).unwrap();
		assert!(kvp1.game_qualifier().is_none());
		assert_eq!(kvp1.key().text(), "MEGALOPOLIS");
		assert_eq!(
			&kvp1
				.string_parts()
				.map(|token| token.text().to_string())
				.collect::<String>(),
			r#""The Omega""#
		);
	}

	#[test]
	fn error_recovery() {
		const SOURCE: &str = r#"
ABDUCTION = ;
[eng]
$ifgame(harmony) HARMS_WAY = "Operation Rescue";
"#;

		let tbuf = crate::scan(SOURCE, Version::default());
		let result = crate::parse(file(), SOURCE, &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);
		let mut ast = ptree.cursor().children();

		assert_eq!(ast.next().unwrap().kind(), Syn::Error);

		let header = ast::Header::cast(ast.next().unwrap()).unwrap();
		let mut header_contents = header.contents();
		assert_eq!(header_contents.next().unwrap().text(), "eng");

		let kvp = ast::KeyValuePair::cast(ast.next().unwrap()).unwrap();
		assert_eq!(kvp.game_qualifier().unwrap().game_id().text(), "harmony");
		assert_eq!(kvp.key().text(), "HARMS_WAY");
		assert_eq!(
			&kvp.string_parts()
				.map(|token| token.text().to_string())
				.collect::<String>(),
			r#""Operation Rescue""#
		);
	}

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
			.map_err(|err| panic!("file I/O failure: {err}"))
			.unwrap();
		let source = String::from_utf8_lossy(&bytes);

		let tbuf = crate::scan(source.as_ref(), Version::default());
		let result = crate::parse(file(), source.as_ref(), &tbuf);
		let ptree: ParseTree = unwrap_parse_tree(result);

		assert_no_errors(&ptree);
	}
}
