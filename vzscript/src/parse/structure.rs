//! Structural items; classes, mixins, structs, unions.

use doomfront::parser::{OpenMark, Parser};

use crate::Syn;

use super::common::*;

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

		while !p.at(Syn::BraceR) && !p.eof() {
			struct_innard(p);
		}

		p.close(mark, Syn::ClassDef);
	}
}

pub(super) struct StructDef;

impl StructDef {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::KwStruct];

	pub(super) fn parse(p: &mut Parser<Syn>, mark: OpenMark) {
		p.expect(Syn::KwStruct, Syn::KwStruct, &["`struct`"]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		trivia_0plus(p);

		while !p.at(Syn::BraceR) && !p.eof() {
			struct_innard(p);
		}

		p.close(mark, Syn::StructDef);
	}
}

pub(super) struct UnionDef;

impl UnionDef {
	pub(super) const FIRST_SET: &[Syn] = &[Syn::KwUnion];

	pub(super) fn parse(p: &mut Parser<Syn>, mark: OpenMark) {
		p.expect(Syn::KwUnion, Syn::KwUnion, &["`union`"]);
		trivia_0plus(p);
		p.expect(Syn::Ident, Syn::Ident, &["an identifier"]);
		trivia_0plus(p);

		while !p.at(Syn::BraceR) && !p.eof() {
			union_innard(p);
		}

		p.close(mark, Syn::UnionDef);
	}
}

fn struct_innard(p: &mut Parser<Syn>) {
	let mark = p.open();

	Attribute::parse_0plus(p);

	let quals = p.open();

	while p.eat_any(MEMBER_QUAL_TOKENS) && !p.eof() {
		trivia_0plus(p);
	}

	p.close(quals, Syn::MemberQuals);

	match p.nth(0) {
		Syn::KwFunction => {
			super::item::FuncDecl::parse(p, mark);
		}
		Syn::Ident => {
			field_decl(p, mark);
		}
		other => {
			p.advance_err_and_close(
				mark,
				other,
				Syn::Error,
				&[
					// TODO
				],
			);
		}
	}
}

fn union_innard(p: &mut Parser<Syn>) {
	let mark = p.open();

	Attribute::parse_0plus(p);

	let quals = p.open();

	while p.eat_any(MEMBER_QUAL_TOKENS) && !p.eof() {
		trivia_0plus(p);
	}

	p.close(quals, Syn::MemberQuals);

	match p.nth(0) {
		Syn::KwFunction => {
			super::item::FuncDecl::parse(p, mark);
		}
		Syn::Ident => {
			p.advance(Syn::Ident);
			trivia_0plus(p);
			p.expect(Syn::BraceL, Syn::BraceR, &["`{`"]);
			trivia_0plus(p);

			while !p.at(Syn::BraceR) && !p.eof() {
				let m = p.open();
				doc_comments(p);
				Attribute::parse_0plus(p);
				field_decl(p, m);
				trivia_0plus(p);
			}

			p.expect(Syn::BraceR, Syn::BraceR, &["`}`"]);
			p.close(mark, Syn::UnionVariant);
		}
		other => {
			p.advance_err_and_close(
				mark,
				other,
				Syn::Error,
				&[
					// TODO
				],
			);
		}
	}
}

fn field_decl(p: &mut Parser<Syn>, mark: OpenMark) {
	p.advance(Syn::Ident);
	trivia_0plus(p);
	TypeSpec::parse(p);
	trivia_0plus(p);
	p.expect(Syn::Semicolon, Syn::Semicolon, &["`;`"]);
	p.close(mark, Syn::FieldDecl);
}

const MEMBER_QUAL_TOKENS: &[(Syn, Syn)] = &[
	(Syn::KwAbstract, Syn::KwAbstract),
	(Syn::KwFinal, Syn::KwFinal),
	(Syn::KwPrivate, Syn::KwPrivate),
	(Syn::KwProtected, Syn::KwProtected),
	(Syn::KwStatic, Syn::KwStatic),
	(Syn::KwOverride, Syn::KwOverride),
	(Syn::KwVirtual, Syn::KwVirtual),
];
