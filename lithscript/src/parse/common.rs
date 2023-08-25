use doomfront::parser::{CloseMark, OpenMark, Parser};

use crate::Syn;

use super::{expr::*, stat::*};

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

pub(super) fn doc_comments(p: &mut Parser<Syn>) {
	while p.eat(Syn::DocComment, Syn::DocComment) {}
}

pub(super) fn block(
	p: &mut Parser<Syn>,
	mark: OpenMark,
	kind: Syn,
	allow_label: bool,
) -> CloseMark {
	if allow_label && p.at_any(BlockLabel::FIRST_SET) {
		BlockLabel::parse(p);
	}

	p.expect(Syn::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	while !p.eof() && !p.at(Syn::BraceR) {
		statement(p);
		trivia_0plus(p);
	}

	p.expect(Syn::BraceR, Syn::BraceR, &["`}`"]);
	p.close(mark, kind)
}

pub(super) struct Annotation;

impl Annotation {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::Pound];

	pub(super) fn parse(p: &mut Parser<Syn>, inner: bool) {
		p.debug_assert_at_any(Self::FIRST_SET);
		let mark = p.open();
		p.advance(Syn::Pound);

		if inner {
			p.expect(Syn::Bang, Syn::Bang, &["`!`"]);
		}

		p.expect(Syn::BracketL, Syn::BracketR, &["`[`"]);
		trivia_0plus(p);

		let idchain = p.open();

		if p.eat(Syn::Dot, Syn::Dot) {
			trivia_0plus(p);
		}

		p.expect(Syn::Ident, Syn::Ident, &["an identifier", "`.`"]);
		trivia_0plus(p);

		while !p.at_any(&[Syn::BracketR, Syn::ParenL]) && !p.eof() {
			p.expect(Syn::Dot, Syn::Dot, &["`.`"]);
			trivia_0plus(p);
			p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		}

		p.close(idchain, Syn::IdentChain);
		trivia_0plus(p);

		if p.at_any(ArgList::FIRST_SET) {
			ArgList::parse(p);
		}

		p.expect(Syn::BracketR, Syn::BracketR, &["`]`"]);
		p.close(mark, Syn::Annotation);
	}
}

/// Common to call expressions and annotations.
pub(super) struct ArgList;

impl ArgList {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::ParenL];

	pub(super) fn parse(p: &mut Parser<Syn>) {
		let mark = p.open();
		p.expect(Syn::ParenL, Syn::ParenL, &["`(`"]);
		trivia_0plus(p);

		while !p.at(Syn::ParenR) && !p.eof() {
			let arg = p.open();

			if p.at_any(&[Syn::Ident, Syn::NameLit]) {
				let peeked = p.find(0, |token| {
					!token.is_trivia() && !matches!(token, Syn::Ident | Syn::NameLit)
				});

				if peeked == Syn::Colon {
					p.advance(p.nth(0));
					trivia_0plus(p);
					p.advance(Syn::Colon);
					trivia_0plus(p);
				}
			}

			Expression::parse(p);
			p.close(arg, Syn::Argument);
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

		trivia_0plus(p);
		p.expect(Syn::ParenR, Syn::ParenR, &["`)`"]);
		p.close(mark, Syn::ArgList);
	}
}

pub(super) struct BlockLabel;

impl BlockLabel {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::Colon2];

	pub(super) fn parse(p: &mut Parser<Syn>) {
		p.expect(Syn::Colon2, Syn::Colon2, &["`::`"]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		trivia_0plus(p);
		p.expect(Syn::Colon2, Syn::Colon2, &["`::`"]);
	}
}

/// "Type specifier". Common to function declarations and parameters.
pub(super) struct TypeSpec;

impl TypeSpec {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::Colon];

	pub(super) fn parse(p: &mut Parser<Syn>) {
		p.expect(Syn::Colon, Syn::Colon, &["`:`"]);
		trivia_0plus(p);
		Expression::parse(p);
	}
}
