//! Structural items; classes, mixins, structs, unions.

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

use super::common::*;

const MEMBER_QUAL_TOKENS: &[Syn] = &[
	Syn::KwAbstract,
	Syn::KwFinal,
	Syn::KwPrivate,
	Syn::KwProtected,
	Syn::KwStatic,
	Syn::KwOverride,
	Syn::KwVirtual,
];

pub(super) struct ClassDef;

impl ClassDef {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::KwClass];

	pub(super) fn parse(p: &mut Parser<Syn>, mark: OpenMark) {
		p.expect(Syn::KwClass, Syn::KwClass, &["`class`"]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		trivia_0plus(p);

		if p.at_any(TypeSpec::FIRST_SET) {
			TypeSpec::parse(p);
			trivia_0plus(p);
		}

		while !p.at(Syn::BraceL) && !p.eof() {}

		while !p.at(Syn::BraceR) && !p.eof() {
			struct_innard(p);
		}

		p.close(mark, Syn::ClassDef);
	}
}

pub(super) struct StructDef;

pub(super) struct UnionDef;

fn struct_innard(p: &mut Parser<Syn>) {}
