//! Function and structure declarations, enums, unions, symbolic constants, et cetera.

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

use super::common::*;

pub(super) fn item(p: &mut Parser<Syn>) {
	let mark = p.open();

	while !p.eof() && p.at_any(Annotation::FIRST_SET) {
		Annotation::parse(p, false);
	}

	trivia_0plus(p);
	doc_comments(p);
	trivia_0plus(p);
	let token = p.nth(0);

	if FuncDecl::FIRST_SET.contains(&token) {
		FuncDecl::parse(p, mark);
	} else if Import::FIRST_SET.contains(&token) {
		Import::parse(p, mark)
	} else {
		p.advance_err_and_close(mark, token, Syn::Error, &["`func`", "`#`"]);
	}
}

#[must_use]
pub(super) fn at_item(p: &mut Parser<Syn>) -> bool {
	p.at_any(Annotation::FIRST_SET) || p.at_any(FuncDecl::FIRST_SET) || p.at_any(Import::FIRST_SET)
}

pub(super) struct FuncDecl;

impl FuncDecl {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::KwFunc];

	pub(super) fn parse(p: &mut Parser<Syn>, mark: OpenMark) {
		p.expect(Syn::KwFunc, Syn::KwFunc, &["`func`"]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		trivia_0plus(p);
		Self::param_list(p);
		trivia_0plus(p);

		if p.at_any(TypeSpec::FIRST_SET) {
			TypeSpec::parse(p);
			trivia_0plus(p);
		}

		if p.eat(Syn::Semicolon, Syn::Semicolon) {
			p.close(mark, Syn::FuncDecl);
			return;
		}

		let body = p.open();
		block(p, body, Syn::FuncBody, false);
		p.close(mark, Syn::FuncDecl);
	}

	fn param_list(p: &mut Parser<Syn>) {
		let mark = p.open();
		p.expect(Syn::ParenL, Syn::ParenR, &["`(`"]);
		trivia_0plus(p);

		while !p.at(Syn::ParenR) && !p.eof() {
			Self::parameter(p);
			trivia_0plus(p);

			match p.nth(0) {
				t @ Syn::Comma => {
					p.advance(t);
					trivia_0plus(p);
				}
				Syn::ParenR => break,
				other => {
					p.advance_with_error(other, &["`,`", "`)`"]);
				}
			}
		}

		p.expect(Syn::ParenR, Syn::ParenR, &["`)`"]);
		p.close(mark, Syn::ParamList);
	}

	fn parameter(p: &mut Parser<Syn>) {
		let mark = p.open();
		p.expect(Syn::Ident, Syn::Ident, &["`an identifier`"]);
		trivia_0plus(p);
		TypeSpec::parse(p);
		p.close(mark, Syn::Parameter);
	}
}

pub(super) struct Import;

impl Import {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::KwImport];

	pub(super) fn parse(p: &mut Parser<Syn>, mark: OpenMark) {
		p.expect(Syn::KwImport, Syn::KwImport, &["`import`"]);
		trivia_0plus(p);
		p.expect(Syn::StringLit, Syn::StringLit, &["a string"]);
		trivia_0plus(p);
		p.expect(Syn::Colon, Syn::Colon, &["`:`"]);
		trivia_0plus(p);

		match p.nth(0) {
			Syn::BraceL => {
				Self::group(p);
			}
			Syn::Ident | Syn::NameLit => {
				Self::single(p);
			}
			Syn::Asterisk => {
				Self::all(p);
			}
			other => {
				p.advance_err_and_close(
					mark,
					other,
					Syn::Error,
					&["`{`", "`*`", "an identifier", "a name literal"],
				);

				return;
			}
		}

		trivia_0plus(p);
		p.expect(Syn::Semicolon, Syn::Semicolon, &["`;`"]);
		p.close(mark, Syn::Import);
	}

	fn single(p: &mut Parser<Syn>) {
		let mark = p.open();

		p.expect_any(
			&[(Syn::Ident, Syn::Ident), (Syn::NameLit, Syn::NameLit)],
			&["an identifier", "a name literal"],
		);

		if p.find(0, |token| !token.is_trivia()) == Syn::ThickArrow {
			p.advance(p.nth(0));
			trivia_0plus(p);
			p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		}

		p.close(mark, Syn::ImportEntry);
	}

	fn group(p: &mut Parser<Syn>) {
		p.debug_assert_at(Syn::BraceL);

		let mark = p.open();
		p.advance(p.nth(0));
		trivia_0plus(p);

		while !p.eof() && !p.at(Syn::BraceR) {
			Self::single(p);
			trivia_0plus(p);

			match p.nth(0) {
				t @ Syn::Comma => {
					p.advance(t);
					trivia_0plus(p);
				}
				Syn::BraceR => break,
				other => {
					p.advance_err_and_close(mark, other, Syn::Error, &["`}`", "`,`"]);
					return;
				}
			}
		}

		p.expect(Syn::BraceR, Syn::BraceR, &["`}`"]);
		p.close(mark, Syn::ImportGroup);
	}

	fn all(p: &mut Parser<Syn>) {
		p.debug_assert_at(Syn::Asterisk);

		let mark = p.open();
		p.advance(p.nth(0));
		trivia_0plus(p);
		p.expect(Syn::ThickArrow, Syn::ThickArrow, &["`=>`"]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		p.close(mark, Syn::ImportEntry);
	}
}
