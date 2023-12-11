//! Parsing functions relevant to other parts of the syntax that don't belong anywhere else.

use doomfront::parser::{CloseMark, OpenMark, Parser};

use crate::Syntax;

/// May or may not build a token tagged with one of the following:
/// - [`Syntax::Whitespace`]
/// - [`Syntax::Comment`]
pub(super) fn trivia(p: &mut Parser<Syntax>) -> bool {
	p.eat_any(&[
		(Syntax::Whitespace, Syntax::Whitespace),
		(Syntax::Comment, Syntax::Comment),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut Parser<Syntax>) {
	while trivia(p) {}
}

pub(super) fn _doc_comments(p: &mut Parser<Syntax>) {
	while p.eat(Syntax::DocComment, Syntax::DocComment) {
		trivia_0plus(p);
	}
}

#[must_use]
pub(super) fn at_annotation(p: &Parser<Syntax>) -> bool {
	p.at(Syntax::Pound)
}

#[must_use]
pub(super) fn at_inner_annotation(p: &Parser<Syntax>) -> bool {
	at_annotation(p) && p.nth(1) == Syntax::Bang
}

pub(super) fn annotation(p: &mut Parser<Syntax>, inner: bool) {
	let mark = p.open();
	p.expect(Syntax::Pound, Syntax::Pound, &[&["TODO"]]);

	if inner {
		p.expect(Syntax::Bang, Syntax::Bang, &[&["TODO"]]);
	}

	p.expect(Syntax::BracketL, Syntax::BracketL, &[&["TODO"]]);
	trivia_0plus(p);
	p.expect(Syntax::Ident, Syntax::Ident, &[&["TODO"]]);
	trivia_0plus(p);

	if p.eat(Syntax::Dot, Syntax::Dot) {
		trivia_0plus(p);
		p.expect(Syntax::Ident, Syntax::Ident, &[&["TODO"]]);
	}

	if p.at(Syntax::ParenL) {
		arg_list(p);
		trivia_0plus(p);
	}

	p.expect(Syntax::BracketR, Syntax::BracketR, &[&["TODO"]]);
	p.close(mark, Syntax::Annotation);
}

pub(super) fn arg_list(p: &mut Parser<Syntax>) {
	let mark = p.open();
	p.expect(Syntax::ParenL, Syntax::ParenL, &[&["TODO"]]);
	trivia_0plus(p);

	if p.eat(Syntax::Dot3, Syntax::Dot3) {
		trivia_0plus(p);
		p.expect(Syntax::ParenR, Syntax::ParenR, &[&["TODO"]]);
		p.close(mark, Syntax::ArgList);
		return;
	}

	while !p.at(Syntax::ParenR) && !p.eof() {
		let arg = p.open();

		if p.at_any(&[Syntax::Ident, Syntax::LitName]) {
			let peeked = p.find(0, |token| {
				!token.is_trivia() && !matches!(token, Syntax::Ident | Syntax::LitName)
			});

			if peeked == Syntax::Colon {
				p.advance(p.nth(0));
				trivia_0plus(p);
				p.advance(Syntax::Colon);
				trivia_0plus(p);
			}
		}

		super::expr(p, true);
		p.close(arg, Syntax::Argument);
		trivia_0plus(p);

		match p.nth(0) {
			t @ Syntax::Comma => {
				p.advance(t);
				trivia_0plus(p);

				if p.eat(Syntax::Dot3, Syntax::Dot3) {
					trivia_0plus(p);
				}
			}
			Syntax::ParenR => break,
			other => {
				p.advance_with_error(other, &[&["`,`", "`)`"]]);
			}
		}
	}

	p.expect(Syntax::ParenR, Syntax::ParenR, &[&["TODO"]]);
	p.close(mark, Syntax::ArgList);
}

pub(super) fn block(
	p: &mut Parser<Syntax>,
	mark: OpenMark,
	kind: Syntax,
	allow_label: bool,
) -> CloseMark {
	if allow_label && at_block_label(p) {
		block_label(p);
	}

	p.expect(Syntax::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.eof() && !p.at(Syntax::BraceR) {
		super::core_element::<false>(p);
		trivia_0plus(p);
	}

	p.expect(Syntax::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(mark, kind)
}

#[must_use]
pub(super) fn at_block_label(p: &Parser<Syntax>) -> bool {
	p.at(Syntax::Colon2)
}

pub(super) fn block_label(p: &mut Parser<Syntax>) {
	let mark = p.open();
	p.expect(Syntax::Colon2, Syntax::Colon2, &[&["`::`"]]);
	trivia_0plus(p);
	p.expect(Syntax::Ident, Syntax::Ident, &[&["an identifier"]]);
	trivia_0plus(p);
	p.expect(Syntax::Colon2, Syntax::Colon2, &[&["`::`"]]);
	p.close(mark, Syntax::BlockLabel);
}

#[must_use]
pub(super) fn at_type_spec(p: &Parser<Syntax>) -> bool {
	p.at(Syntax::Colon)
}

pub(super) fn type_spec(p: &mut Parser<Syntax>, param: bool) {
	let mark = p.open();
	p.expect(Syntax::Colon, Syntax::Colon, &[&["`:`"]]);
	trivia_0plus(p);

	if p.eat(Syntax::KwTypeT, Syntax::KwTypeT) {
		p.close(mark, Syntax::TypeSpec);
		return;
	}

	if param && p.eat(Syntax::KwAnyT, Syntax::KwAnyT) {
		p.close(mark, Syntax::TypeSpec);
		return;
	}

	let _ = super::expr(p, false);
	p.close(mark, Syntax::TypeSpec);
}
