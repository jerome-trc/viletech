use crate::{parser::Parser, zdoom::lex::Token};

use super::syn::Syn;

/// Builds a [`Syn::Root`] node.
pub fn file(p: &mut Parser<Syn>) {
	let root = p.open();

	while !p.eof() {
		if trivia(p) {
			continue;
		}

		if p.at(Token::Ident) {
			definition(p);
		} else {
			p.advance_with_error(
				Syn::from(p.nth(0)),
				&[&[
					"`server` or `user` or `nosave`",
					"`nosave` or `noarchive` or `cheat` or `latch`",
					"whitespace",
					"a comment",
				]],
			);
		}
	}

	p.close(root, Syn::Root);
}

/// Builds a [`Syn::Definition`] node.
pub fn definition(p: &mut Parser<Syn>) {
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
			(Token::KwInt, Syn::KwInt),
			(Token::KwFloat, Syn::KwFloat),
			(Token::KwBool, Syn::KwBool),
			(Token::KwColor, Syn::KwColor),
			(Token::KwString, Syn::KwString),
		],
		&[&["`server` or `user` or `nosave`", "whitespace", "a comment"]],
	);

	trivia_1plus(p);

	p.expect(Token::Ident, Syn::Ident, &[&["an identifier"]]);
	trivia_0plus(p);

	if p.at(Token::Eq) {
		default_def(p);
	}

	trivia_0plus(p);
	p.expect(Token::Semicolon, Syn::Semicolon, &[&["`;`"]]);
	p.close(def, Syn::Definition);
}

fn flag(p: &mut Parser<Syn>) {
	p.expect_any_str_nc(
		&[
			(Token::Ident, "server", Syn::KwServer),
			(Token::Ident, "user", Syn::KwUser),
			(Token::Ident, "nosave", Syn::KwNoSave),
			(Token::Ident, "noarchive", Syn::KwNoArchive),
			(Token::Ident, "cheat", Syn::KwCheat),
			(Token::Ident, "latch", Syn::KwLatch),
		],
		&[&[
			"`server` or `user` or `nosave`",
			"`nosave` or `noarchive` or `cheat` or `latch`",
			"whitespace",
			"a comment",
		]],
	)
}

/// Builds a [`Syn::DefaultDef`] node.
fn default_def(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::Eq);
	let default = p.open();
	p.advance(Syn::Eq);
	trivia_0plus(p);

	p.expect_any(
		&[
			(Token::FloatLit, Syn::FloatLit),
			(Token::IntLit, Syn::IntLit),
			(Token::KwFalse, Syn::FalseLit),
			(Token::KwTrue, Syn::TrueLit),
			(Token::StringLit, Syn::StringLit),
		],
		&[&[
			"an integer",
			"a floating-point number",
			"a string",
			"`false` or `true`",
		]],
	);

	p.close(default, Syn::DefaultDef);
}

fn trivia(p: &mut Parser<Syn>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syn::Whitespace),
		(Token::Comment, Syn::Comment),
		(Token::RegionStart, Syn::RegionStart),
		(Token::RegionEnd, Syn::RegionEnd),
	])
}

fn trivia_0plus(p: &mut Parser<Syn>) {
	while trivia(p) {}
}

fn trivia_1plus(p: &mut Parser<Syn>) {
	p.expect_any(
		&[
			(Token::Whitespace, Syn::Whitespace),
			(Token::Comment, Syn::Comment),
			(Token::RegionStart, Syn::RegionStart),
			(Token::RegionEnd, Syn::RegionEnd),
		],
		&[&["whitespace or a comment (one or more)"]],
	);

	trivia_0plus(p)
}

#[cfg(test)]
mod test {
	use rowan::ast::AstNode;

	use crate::zdoom::{
		self,
		cvarinfo::{ast, ParseTree},
	};

	use super::*;

	#[test]
	fn smoke() {
		const SOURCE: &str = r#"

// Rue des Acacias
server int egghead_roundabout;
user float acidSurge=0.4	;
cheat noarchive nosave string /* comment? */ BONELESS_VENTURES = "Welcome to the Company !";

	"#;

		let mut parser = Parser::new(SOURCE, zdoom::lex::Context::NON_ZSCRIPT);
		file(&mut parser);
		let (root, errors) = parser.finish();
		let ptree: ParseTree = ParseTree::new(root, vec![]);
		assert!(errors.is_empty());

		let cvars: Vec<_> = ptree
			.cursor()
			.children()
			.map(|c| ast::Definition::cast(c).unwrap())
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
		assert_eq!(default_1.token().float().unwrap().unwrap(), 0.4);
		assert_eq!(
			default_2.token().string().unwrap(),
			"Welcome to the Company !"
		);
	}

	#[test]
	fn err_recovery() {
		const SOURCE: &str = r#"

	server int theumpteenthcircle = ;
	user float ICEANDFIRE3 = 0.4;

	"#;

		let mut parser = Parser::new(SOURCE, zdoom::lex::Context::NON_ZSCRIPT);
		file(&mut parser);
		let (root, errors) = parser.finish();
		let ptree: ParseTree = ParseTree::new(root, vec![]);
		assert_eq!(errors.len(), 1);

		let cvar = ast::Definition::cast(ptree.cursor().last_child().unwrap()).unwrap();
		assert_eq!(cvar.name().text(), "ICEANDFIRE3");
	}
}
