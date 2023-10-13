//! Parsing functions relevant to other parts of the syntax that don't belong anywhere else.

use doomfront::parser::{CloseMark, OpenMark, Parser};

use crate::Syn;

/// May or may not build a token tagged with one of the following:
/// - [`Syn::Whitespace`]
/// - [`Syn::Comment`]
pub(super) fn trivia(p: &mut Parser<Syn>) -> bool {
	p.eat_any(&[
		(Syn::Whitespace, Syn::Whitespace),
		(Syn::Comment, Syn::Comment),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut Parser<Syn>) {
	while trivia(p) {}
}

pub(super) fn _doc_comments(p: &mut Parser<Syn>) {
	while p.eat(Syn::DocComment, Syn::DocComment) {
		trivia_0plus(p);
	}
}

#[must_use]
pub(super) fn at_annotation(p: &Parser<Syn>) -> bool {
	p.at(Syn::Pound)
}

#[must_use]
pub(super) fn at_inner_annotation(p: &Parser<Syn>) -> bool {
	at_annotation(p) && p.nth(1) == Syn::Bang
}

pub(super) fn annotation(p: &mut Parser<Syn>, inner: bool) {
	let mark = p.open();
	p.expect(Syn::Pound, Syn::Pound, &[&["TODO"]]);

	if inner {
		p.expect(Syn::Bang, Syn::Bang, &[&["TODO"]]);
	}

	p.expect(Syn::BracketL, Syn::BracketL, &[&["TODO"]]);
	trivia_0plus(p);
	p.expect(Syn::Ident, Syn::Ident, &[&["TODO"]]);
	trivia_0plus(p);

	if p.eat(Syn::Dot, Syn::Dot) {
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &[&["TODO"]]);
	}

	if p.at(Syn::ParenL) {
		arg_list(p);
		trivia_0plus(p);
	}

	p.expect(Syn::BracketR, Syn::BracketR, &[&["TODO"]]);
	p.close(mark, Syn::Annotation);
}

pub(super) fn arg_list(p: &mut Parser<Syn>) {
	let mark = p.open();
	p.expect(Syn::ParenL, Syn::ParenL, &[&["TODO"]]);

	while !p.at(Syn::ParenR) && !p.eof() {
		let arg = p.open();

		if p.at_any(&[Syn::Ident, Syn::LitName]) {
			let peeked = p.find(0, |token| {
				!token.is_trivia() && !matches!(token, Syn::Ident | Syn::LitName)
			});

			if peeked == Syn::Colon {
				p.advance(p.nth(0));
				trivia_0plus(p);
				p.advance(Syn::Colon);
				trivia_0plus(p);
			}
		}

		super::expr(p);
		p.close(arg, Syn::Argument);
		trivia_0plus(p);

		match p.nth(0) {
			t @ Syn::Comma => {
				p.advance(t);
				trivia_0plus(p);
			}
			Syn::ParenR => break,
			other => {
				p.advance_with_error(other, &[&["`,`", "`)`"]]);
			}
		}
	}

	p.expect(Syn::ParenR, Syn::ParenR, &[&["TODO"]]);
	p.close(mark, Syn::ArgList);
}

pub(super) fn block(
	p: &mut Parser<Syn>,
	mark: OpenMark,
	kind: Syn,
	allow_label: bool,
) -> CloseMark {
	if allow_label && at_block_label(p) {
		block_label(p);
	}

	p.expect(Syn::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.eof() && !p.at(Syn::BraceR) {
		super::core_element::<false>(p);
		trivia_0plus(p);
	}

	p.expect(Syn::BraceR, Syn::BraceR, &[&["`}`"]]);
	p.close(mark, kind)
}

#[must_use]
pub(super) fn at_block_label(p: &Parser<Syn>) -> bool {
	p.at(Syn::Colon2)
}

pub(super) fn block_label(p: &mut Parser<Syn>) {
	let mark = p.open();
	p.expect(Syn::Colon2, Syn::Colon2, &[&["`::`"]]);
	trivia_0plus(p);
	p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
	trivia_0plus(p);
	p.expect(Syn::Colon2, Syn::Colon2, &[&["`::`"]]);
	p.close(mark, Syn::BlockLabel);
}

#[must_use]
pub(super) fn at_type_spec(p: &Parser<Syn>) -> bool {
	p.at(Syn::Colon)
}

pub(super) fn type_spec(p: &mut Parser<Syn>) {
	let mark = p.open();
	p.expect(Syn::Colon, Syn::Colon, &[&["`:`"]]);
	trivia_0plus(p);
	let _ = super::expr(p);
	p.close(mark, Syn::TypeSpec);
}
