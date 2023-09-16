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

	p.expect(Syn::BraceL, Syn::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.eof() && !p.at(Syn::BraceR) {
		super::core_element::<false>(p);
		trivia_0plus(p);
	}

	p.expect(Syn::BraceR, Syn::BraceR, &[&["`}`"]]);
	p.close(mark, kind)
}

/// Common to call expressions, annotations, and attributes.
pub(super) struct ArgList;

impl ArgList {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::ParenL];

	pub(super) fn parse(p: &mut Parser<Syn>) {
		let mark = p.open();
		p.expect(Syn::ParenL, Syn::ParenL, &[&["`(`"]]);
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
					p.advance_with_error(other, &[&["`,`", "`)`"]]);
				}
			}
		}

		trivia_0plus(p);
		p.expect(Syn::ParenR, Syn::ParenR, &[&["`)`"]]);
		p.close(mark, Syn::ArgList);
	}
}

pub(super) struct Attribute;

impl Attribute {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::PoundBracketL];

	pub(super) fn parse(p: &mut Parser<Syn>) {
		p.debug_assert_at_any(Self::FIRST_SET);
		let mark = p.open();
		p.advance(Syn::PoundBracketL);

		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
		trivia_0plus(p);

		if p.at_any(ArgList::FIRST_SET) {
			trivia_0plus(p);
			ArgList::parse(p);
		}

		p.expect(Syn::BracketR, Syn::BracketR, &[&["`]`"]]);
		p.close(mark, Syn::Attribute);
	}

	pub(super) fn parse_0plus(p: &mut Parser<Syn>) {
		while p.at_any(Self::FIRST_SET) && !p.eof() {
			Self::parse(p);
			trivia_0plus(p);
		}
	}
}

pub(super) struct Annotation;

impl Annotation {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::Pound];

	pub(super) fn parse(p: &mut Parser<Syn>) {
		p.debug_assert_at_any(Self::FIRST_SET);
		let mark = p.open();
		p.advance(Syn::Pound);
		p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);

		if p.find(0, |token| !token.is_trivia()) == Syn::ParenL {
			trivia_0plus(p);
			ArgList::parse(p);
		}

		p.close(mark, Syn::Annotation);
	}
}

pub(super) struct BlockLabel;

impl BlockLabel {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::Colon2];

	pub(super) fn parse(p: &mut Parser<Syn>) {
		let mark = p.open();
		p.expect(Syn::Colon2, Syn::Colon2, &[&["`::`"]]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &[&["an identifier"]]);
		trivia_0plus(p);
		p.expect(Syn::Colon2, Syn::Colon2, &[&["`::`"]]);
		p.close(mark, Syn::BlockLabel);
	}
}

pub(super) fn name_chain(p: &mut Parser<Syn>) {
	let mark = p.open();

	if p.eat(Syn::Dot, Syn::Dot) {
		trivia_0plus(p);
	}

	p.expect_any(
		&[(Syn::Ident, Syn::Ident), (Syn::NameLit, Syn::NameLit)],
		&[&["an identifier", "a name literal"]],
	);

	while p.find(0, |token| !token.is_trivia()) == Syn::Dot {
		trivia_0plus(p);
		p.advance(Syn::Dot);
		p.expect_any(
			&[(Syn::Ident, Syn::Ident), (Syn::NameLit, Syn::NameLit)],
			&[&["an identifier", "a name literal"]],
		);
	}

	p.close(mark, Syn::NameChain);
}

/// "Type specifier". Common to
/// - class definitions (inheritance specification)
/// - enum definitions (underlying type specification)
/// - field declarations
/// - function declarations (return types, parameters)
pub(super) struct TypeSpec;

impl TypeSpec {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::Colon];

	pub(super) fn parse(p: &mut Parser<Syn>) {
		let mark = p.open();
		p.expect(Syn::Colon, Syn::Colon, &[&["`:`"]]);
		trivia_0plus(p);

		if !p.eat(Syn::KwAuto, Syn::KwAuto) {
			let _ = Expression::parse(p);
		}

		p.close(mark, Syn::TypeSpec);
	}
}
