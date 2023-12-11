use crate::{parser::Parser, zdoom::lex::Token};

use super::syntax::Syntax;

/// Builds a [`Syntax::Root`] node.
pub fn file(p: &mut Parser<Syntax>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		if p.at(Token::Ident) {
			definition(p);
		} else {
			p.advance_with_error(
				Syntax::from(p.nth(0)),
				&[&[
					"`server` or `user` or `nosave`",
					"`nosave` or `noarchive` or `cheat` or `latch`",
					"whitespace",
					"a comment",
				]],
			);
		}
	}

	p.close(root, Syntax::Root);
}

/// Builds a [`Syntax::Definition`] node.
pub fn definition(p: &mut Parser<Syntax>) {
	let def = p.open();
	flag(p);
	trivia_1plus(p);

	while !p.at_any(&[
		Token::KwInt,
		Token::KwFloat,
		Token::KwBool,
		Token::KwColor,
		Token::KwString,
	]) && !p.eof()
	{
		flag(p);
		trivia_1plus(p);
	}

	p.expect_any(
		&[
			(Token::KwInt, Syntax::KwInt),
			(Token::KwFloat, Syntax::KwFloat),
			(Token::KwBool, Syntax::KwBool),
			(Token::KwColor, Syntax::KwColor),
			(Token::KwString, Syntax::KwString),
		],
		&[&["`server` or `user` or `nosave`", "whitespace", "a comment"]],
	);

	trivia_1plus(p);

	p.expect(Token::Ident, Syntax::Ident, &[&["an identifier"]]);
	trivia_0plus(p);

	if p.at(Token::Eq) {
		default_def(p);
	}

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
	p.close(def, Syntax::Definition);
}

fn flag(p: &mut Parser<Syntax>) {
	p.expect_any_str_nc(
		&[
			(Token::Ident, "server", Syntax::KwServer),
			(Token::Ident, "user", Syntax::KwUser),
			(Token::Ident, "nosave", Syntax::KwNoSave),
			(Token::Ident, "noarchive", Syntax::KwNoArchive),
			(Token::Ident, "cheat", Syntax::KwCheat),
			(Token::Ident, "latch", Syntax::KwLatch),
		],
		&[&[
			"`server` or `user` or `nosave`",
			"`nosave` or `noarchive` or `cheat` or `latch`",
			"whitespace",
			"a comment",
		]],
	)
}

/// Builds a [`Syntax::DefaultDef`] node.
fn default_def(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::Eq);
	let default = p.open();
	p.advance(Syntax::Eq);
	trivia_0plus(p);

	p.expect_any(
		&[
			(Token::FloatLit, Syntax::FloatLit),
			(Token::IntLit, Syntax::IntLit),
			(Token::KwFalse, Syntax::FalseLit),
			(Token::KwTrue, Syntax::TrueLit),
			(Token::StringLit, Syntax::StringLit),
		],
		&[&[
			"an integer",
			"a floating-point number",
			"a string",
			"`false` or `true`",
		]],
	);

	p.close(default, Syntax::DefaultDef);
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

fn trivia_1plus(p: &mut Parser<Syntax>) {
	p.expect_any(
		&[
			(Token::Whitespace, Syntax::Whitespace),
			(Token::Comment, Syntax::Comment),
			(Token::DocComment, Syntax::Comment),
			(Token::RegionStart, Syntax::RegionStart),
			(Token::RegionEnd, Syntax::RegionEnd),
		],
		&[&["whitespace or a comment (one or more)"]],
	);

	trivia_0plus(p)
}

#[cfg(test)]
mod test {
	use rowan::ast::AstNode;

	use crate::{
		testing::*,
		zdoom::{
			self,
			cvarinfo::{ast, ParseTree},
		},
	};

	use super::*;

	#[test]
	fn smoke() {
		const SAMPLE: &str = r#"

// Rue des Acacias
server int egghead_roundabout;
user float acidSurge=0.4	;
cheat noarchive nosave string /* comment? */ BONELESS_VENTURES = "Welcome to the Company !";

	"#;

		let ptree: ParseTree = crate::parse(SAMPLE, file, zdoom::lex::Context::NON_ZSCRIPT);

		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());

		let cvars: Vec<_> = ptree
			.cursor()
			.children()
			.map(|c| ast::Definition::cast(c).unwrap())
			.collect();

		assert_eq!(cvars[0].name().text(), "egghead_roundabout");
		assert_eq!(cvars[1].name().text(), "acidSurge");
		assert_eq!(cvars[2].name().text(), "BONELESS_VENTURES");

		assert_eq!(cvars[0].type_spec().kind(), Syntax::KwInt);
		assert_eq!(cvars[1].type_spec().kind(), Syntax::KwFloat);
		assert_eq!(cvars[2].type_spec().kind(), Syntax::KwString);

		let default_0 = cvars[0].default();
		let default_1 = cvars[1].default().unwrap();
		let default_2 = cvars[2].default().unwrap();

		assert_eq!(default_0, None);
		assert_eq!(default_1.token().float().unwrap().unwrap(), 0.4);
		assert_eq!(
			default_2.token().string().unwrap(),
			"Welcome to the Company !"
		);
	}

	#[test]
	fn err_recovery() {
		const SAMPLE: &str = r#"

	server int theumpteenthcircle = ;
	user float ICEANDFIRE3 = 0.4;

	"#;

		let ptree: ParseTree = crate::parse(SAMPLE, file, zdoom::lex::Context::NON_ZSCRIPT);

		assert_eq!(ptree.errors().len(), 1);
		prettyprint_maybe(ptree.cursor());

		let cvar = ast::Definition::cast(ptree.cursor().last_child().unwrap()).unwrap();
		assert_eq!(cvar.name().text(), "ICEANDFIRE3");
	}
}
