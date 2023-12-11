use crate::{parser::Parser, zdoom::lex::Token};

use super::Syntax;

/// Builds a [`Syntax::Root`] node.
pub fn file(p: &mut Parser<Syntax>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		if p.at_if(|t| matches!(t, Token::Ident | Token::Dollar) || t.is_keyword()) {
			key_val_pair(p);
		} else if p.at(Token::BracketL) {
			header(p);
		} else {
			p.advance_with_error(
				Syntax::from(p.nth(0)),
				&[&[
					"a key-value pair (`$` or an identifier)",
					"a header (`[`)",
					"whitespace",
					"a comment",
				]],
			);
		}
	}

	p.close(root, Syntax::Root);
}

/// Builds a [`Syntax::KeyValuePair`] node.
pub fn key_val_pair(p: &mut Parser<Syntax>) {
	p.assert_at_if(|t| matches!(t, Token::Ident | Token::Dollar));
	let kvp = p.open();

	if p.at(Token::Dollar) {
		ifgame(p);
	}

	p.expect_if(
		|t| t == Token::Ident || t.is_keyword(),
		Syntax::Ident,
		&[&["an identifier"]],
	);
	trivia_0plus(p);
	p.expect(Token::Eq, Syntax::Eq, &[&["`=`"]]);
	string(p);

	loop {
		if p.at_any(&[Token::Eof, Token::Semicolon, Token::Ident, Token::BracketL]) {
			break;
		}

		string(p);
	}

	trivia_0plus(p);
	p.eat(Token::Semicolon, Syntax::Semicolon);
	p.close(kvp, Syntax::KeyValuePair);
}

fn string(p: &mut Parser<Syntax>) {
	trivia_0plus(p);
	p.expect(Token::StringLit, Syntax::StringLit, &[&["a string"]]);
}

/// Builds a [`Syntax::GameQualifier`] node.
fn ifgame(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::Dollar);
	let ifgame = p.open();
	p.advance(Syntax::Dollar);
	trivia_0plus(p);
	p.expect_str_nc(Token::Ident, "ifgame", Syntax::KwIfGame, &[&["`ifgame`"]]);
	trivia_0plus(p);
	p.expect(Token::ParenL, Syntax::ParenL, &[&["`(`"]]);
	trivia_0plus(p);
	p.expect_if(
		|t| t == Token::Ident || t.is_keyword(),
		Syntax::Ident,
		&[&["an identifier"]],
	);
	trivia_0plus(p);
	p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
	trivia_0plus(p);
	p.close(ifgame, Syntax::GameQualifier);
}

/// Builds a [`Syntax::Header`] node.
pub fn header(p: &mut Parser<Syntax>) {
	p.assert_at(Token::BracketL);
	let header = p.open();
	p.advance(Syntax::BracketL);

	while !p.at(Token::BracketR) && !p.eof() {
		if trivia(p) {
			continue;
		}

		let token = p.nth(0);

		if token == Token::Ident || token.is_keyword() {
			p.advance(Syntax::Ident);
			continue;
		}

		match token {
			Token::Tilde => p.advance(Syntax::Tilde),
			Token::KwDefault => p.advance(Syntax::KwDefault),
			Token::Asterisk => p.advance(Syntax::Asterisk),
			t => {
				if p.at_any(&[Token::Dollar]) {
					break;
				}

				return p.advance_with_error(
					Syntax::from(t),
					&[&["`~`", "`*`", "`default`", "an identifier"]],
				);
			}
		}
	}

	p.expect(Token::BracketR, Syntax::BracketR, &[&["`]`"]]);
	p.close(header, Syntax::Header);
}

fn trivia(p: &mut Parser<Syntax>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syntax::Whitespace),
		(Token::Comment, Syntax::Comment),
		(Token::DocComment, Syntax::Comment),
		(Token::RegionStart, Syntax::RegionStart),
		(Token::RegionEnd, Syntax::RegionEnd),
	])
}

fn trivia_0plus(p: &mut Parser<Syntax>) {
	while trivia(p) {}
}

#[cfg(test)]
mod test {
	use rowan::ast::AstNode;

	use crate::{
		testing::*,
		zdoom::{
			self,
			language::{ast, ParseTree},
		},
	};

	use super::*;

	#[test]
	fn smoke() {
		const SAMPLE: &str = r#"
[enu * ~ default]

$ifgame(harmony) THE_UNDERWATER_LAB = "Echidna";
MEGALOPOLIS = "The Omega";
"#;

		let ptree: ParseTree = crate::parse(SAMPLE, file, zdoom::lex::Context::NON_ZSCRIPT);
		assert_no_errors(&ptree);
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
		const SAMPLE: &str = r#"
ABDUCTION = ;
[eng
$ifgame(harmony) HARMS_WAY = "Operation Rescue";
"#;

		let ptree: ParseTree = crate::parse(SAMPLE, file, zdoom::lex::Context::NON_ZSCRIPT);
		assert_eq!(ptree.errors.len(), 2);
		prettyprint_maybe(ptree.cursor());
		let mut ast = ptree.cursor().children();

		let kvp = ast::KeyValuePair::cast(ast.next().unwrap()).unwrap();
		assert_eq!(kvp.key().text(), "ABDUCTION");
		assert_eq!(kvp.string_parts().count(), 0);

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
	#[ignore]
	fn with_sample_data() {
		let sample = match read_sample_data("DOOMFRONT_LANGUAGE_SAMPLE") {
			Ok((_, s)) => s,
			Err(err) => {
				eprintln!("Skipping LANGUAGE sample data-based unit test. Reason: {err}");
				return;
			}
		};

		let ptree: ParseTree = crate::parse(&sample, file, zdoom::lex::Context::NON_ZSCRIPT);
		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}
}
