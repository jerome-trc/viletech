//! Parsers for definitions for classes, mixin classes, and structs.

use crate::{
	parser::Parser,
	zdoom::{zscript::Syntax, Token},
};

use super::*;

/// Builds a [`Syntax::ClassDef`] node.
///
/// Returns `true` if a full-file class was parsed.
#[must_use]
pub fn class_def(p: &mut Parser<Syntax>) -> bool {
	p.debug_assert_at_any(&[Token::KwClass, Token::DocComment]);
	let classdef = p.open();
	doc_comments(p);
	class_head(p);

	if p.eat(Token::Semicolon, Syntax::Semicolon) {
		trivia_0plus(p);

		while !p.eof() {
			class_innard::<false>(p);
			trivia_0plus(p);
		}

		p.close(classdef, Syntax::ClassDef);
		return true;
	}

	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		class_innard::<false>(p);
		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(classdef, Syntax::ClassDef);
	false
}

fn class_head(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwClass);
	let mark = p.open();
	p.advance(Syntax::KwClass);
	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);

	if p.at(Token::Colon) {
		let inherit = p.open();
		p.advance(Syntax::Colon);
		trivia_0plus(p);
		ident_lax(p);
		p.close(inherit, Syntax::InheritSpec);
	}

	trivia_0plus(p);

	while !p.at_any(&[Token::BraceL, Token::Semicolon]) && !p.eof() {
		match p.nth(0) {
			Token::KwReplaces => {
				let replaces = p.open();
				p.advance(Syntax::KwReplaces);
				trivia_0plus(p);
				ident_lax(p);
				p.close(replaces, Syntax::ReplacesClause);
			}
			Token::KwAbstract => p.advance(Syntax::KwAbstract),
			Token::KwPlay => p.advance(Syntax::KwPlay),
			Token::KwUi => p.advance(Syntax::KwUi),
			Token::KwNative => p.advance(Syntax::KwNative),
			Token::KwVersion => version_qual(p),
			other => p.advance_with_error(
				Syntax::from(other),
				&[&[
					"`abstract`",
					"`native`",
					"`play`",
					"`replaces`",
					"`ui`",
					"`version`",
				]],
			),
		}

		trivia_0plus(p);
	}

	p.close(mark, Syntax::ClassHead);
}

/// Builds a [`Syntax::MixinClassDef`] node.
pub fn mixin_class_def(p: &mut Parser<Syntax>) {
	p.debug_assert_at_any(&[Token::KwMixin, Token::DocComment]);
	let mixindef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwMixin);
	p.advance(Syntax::KwMixin);
	trivia_0plus(p);
	p.expect(Token::KwClass, Syntax::KwClass, &[&["`class`"]]);
	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		class_innard::<true>(p);
		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(mixindef, Syntax::MixinClassDef);
}

/// Builds a [`Syntax::StructDef`] node.
pub fn struct_def(p: &mut Parser<Syntax>) {
	p.debug_assert_at_any(&[Token::KwStruct, Token::DocComment]);
	let structdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwStruct);
	p.advance(Syntax::KwStruct);
	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);
	let quals = p.open();

	while !p.at(Token::BraceL) && !p.eof() {
		match p.nth(0) {
			Token::KwPlay => p.advance(Syntax::KwPlay),
			Token::KwUi => p.advance(Syntax::KwUi),
			Token::KwNative => p.advance(Syntax::KwNative),
			Token::KwClearScope => p.advance(Syntax::KwClearScope),
			Token::KwVersion => version_qual(p),
			other => p.advance_with_error(
				Syntax::from(other),
				&[&["`clearscope`", "`native`", "`play`", "`ui`", "`version`"]],
			),
		}

		trivia_0plus(p);
	}

	p.close(quals, Syntax::StructQuals);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		struct_innard(p);
		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);

	if p.find(0, |token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syntax::Semicolon);
	}

	p.close(structdef, Syntax::StructDef);
}

/// Builds a [`Syntax::ClassExtend`] or [`Syntax::StructExtend`] node.
pub fn class_or_struct_extend(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::KwExtend);
	let extension = p.open();
	p.advance(Syntax::KwExtend);
	trivia_0plus(p);

	let node_syn = match p.nth(0) {
		Token::KwClass => {
			p.advance(Syntax::KwClass);
			Syntax::ClassExtend
		}
		Token::KwStruct => {
			p.advance(Syntax::KwStruct);
			Syntax::StructExtend
		}
		other => {
			p.advance_err_and_close(
				extension,
				Syntax::from(other),
				Syntax::Unknown,
				&[&["`class`", "`struct`"]],
			);
			return;
		}
	};

	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syntax::BraceL, &[&["`{`"]]);
	trivia_0plus(p);

	if node_syn == Syntax::ClassExtend {
		while !p.at(Token::BraceR) && !p.eof() {
			class_innard::<false>(p);
			trivia_0plus(p);
		}
	} else if node_syn == Syntax::StructExtend {
		while !p.at(Token::BraceR) && !p.eof() {
			struct_innard(p);
			trivia_0plus(p);
		}
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syntax::BraceR, &[&["`}`"]]);
	p.close(extension, node_syn);
}

// Innards /////////////////////////////////////////////////////////////////////

fn class_innard<const MIXIN: bool>(p: &mut Parser<Syntax>) {
	let token = p.find(0, |token| !token.is_trivia());

	if token == Token::KwStatic && p.find(1, |token| !token.is_trivia()) == Token::KwConst {
		static_const_stat(p);
		return;
	}

	if in_type_ref_first_set(token) || in_decl_qual_first_set(token) {
		member_decl(p);
		return;
	}

	match token {
		Token::KwConst => {
			const_def(p);
			return;
		}
		Token::KwEnum => {
			enum_def(p);
			return;
		}
		Token::KwProperty => {
			property_def(p);
			return;
		}
		Token::KwFlagDef => {
			flag_def(p);
			return;
		}
		Token::KwStruct => {
			struct_def(p);
			return;
		}
		_ => {}
	}

	if p.at(Token::DocComment) {
		// Class innards outside this set can not start with a doc comment.
		p.advance(Syntax::Comment);
		return;
	}

	if p.at(Token::KwMixin) && !MIXIN {
		let mixin = p.open();
		p.advance(Syntax::KwMixin);
		trivia_0plus(p);
		ident_lax(p);
		trivia_0plus(p);
		p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
		p.close(mixin, Syntax::MixinStat);
		return;
	}

	match token {
		Token::KwDefault => {
			default_block(p);
			return;
		}
		Token::KwStates => {
			states_block(p);
			return;
		}
		_ => {}
	}

	const EXPECTED: &[&str] = &[
		"a type name",
		"`const` or `enum` or `struct`",
		"`states` or `default`",
		"`property` or `flagdef`",
		"`play` or `ui` or `virtualscope` or `clearscope`",
		"`deprecated` or `version`",
		"`abstract` or `final` or `override` or `virtual`",
		"`private` or `protected`",
		"`internal` or `meta` or `native` or `transient`",
		"`static` or `readonly`",
		"`vararg`",
	];

	p.advance_with_error(
		Syntax::from(p.nth(0)),
		if !MIXIN {
			&[EXPECTED, &["`mixin`"]]
		} else {
			&[EXPECTED]
		},
	)
}

fn struct_innard(p: &mut Parser<Syntax>) {
	let token = p.find(0, |token| !token.is_trivia());

	if token == Token::KwStatic && p.find(1, |token| !token.is_trivia()) == Token::KwConst {
		static_const_stat(p);
		return;
	}

	if in_type_ref_first_set(token) || in_decl_qual_first_set(token) {
		member_decl(p);
		trivia_0plus(p);
		return;
	}

	match token {
		Token::KwConst => const_def(p),
		Token::KwEnum => enum_def(p),
		other => p.advance_with_error(
			Syntax::from(other),
			&[&[
				"a type name",
				"`const` or `enum`",
				"`play` or `ui` or `virtualscope` or `clearscope`",
				"`deprecated` or `version`",
				"`abstract` or `final` or `override` or `virtual`",
				"`private` or `protected`",
				"`internal` or `meta` or `native` or `transient`",
				"`static` or `readonly`",
				"`vararg`",
			]],
		),
	}

	trivia_0plus(p);
}

/// Builds a [`Syntax::FieldDecl`] or [`Syntax::FunctionDecl`] node.
pub(super) fn member_decl(p: &mut Parser<Syntax>) {
	let member = p.open();
	doc_comments(p);
	let quals = p.open();

	while p.at_if(in_decl_qual_first_set) && !p.eof() {
		match p.nth(0) {
			Token::KwDeprecated => deprecation_qual(p),
			Token::KwVersion => version_qual(p),
			Token::KwAction => {
				let action = p.open();
				p.advance(Syntax::KwAction);

				if p.find(0, |token| !token.is_trivia()) == Token::ParenL {
					trivia_0plus(p);
					states_usage(p);
				}

				p.close(action, Syntax::ActionQual);
			}
			Token::KwReadOnly => {
				if p.find(1, |token| !token.is_trivia()) == Token::AngleL {
					break; // We are likely looking at a `readonly<T>` return type.
				}

				p.advance(Syntax::KwReadOnly);
			}
			other => p.advance(Syntax::from(other)),
		}

		trivia_0plus(p);
	}

	p.close(quals, Syntax::MemberQuals);
	let rettypes = p.open();
	type_ref(p);

	while p.find(0, |token| !token.is_trivia()) == Token::Comma {
		trivia_0plus(p);
		p.advance(Syntax::Comma);
		trivia_0plus(p);
		type_ref(p);
	}

	trivia_0plus(p);

	if !p.at_if(is_ident_lax) {
		p.cancel(rettypes);
		p.advance_err_and_close(
			member,
			Syntax::from(p.nth(0)),
			Syntax::Error,
			&[&["an identifier"]],
		);
		return;
	}

	let peeked = p.find(0, |token| !token.is_trivia() && !is_ident_lax(token));

	match peeked {
		Token::BracketL | Token::Comma => {
			p.cancel(rettypes);
			trivia_0plus(p);

			while !p.at(Token::Semicolon) && !p.eof() {
				var_name(p);

				if p.find(0, |token| !token.is_trivia()) == Token::Comma {
					trivia_0plus(p);
					p.advance(Syntax::Comma);
					trivia_0plus(p);
				} else {
					break;
				}
			}

			p.expect(Token::Semicolon, Syntax::Semicolon, &[&["`;`"]]);
			p.close(member, Syntax::FieldDecl);
		}
		Token::ParenL => {
			p.close(rettypes, Syntax::ReturnTypes);
			trivia_0plus(p);
			ident_lax(p);
			trivia_0plus(p);
			param_list(p);
			trivia_0plus(p);

			if p.eat(Token::KwConst, Syntax::KwConst) {
				trivia_0plus(p);
			}

			if p.eat(Token::Semicolon, Syntax::Semicolon) {
				p.close(member, Syntax::FunctionDecl);
				return;
			}

			compound_stat(p);
			p.close(member, Syntax::FunctionDecl);
		}
		Token::Semicolon => {
			p.cancel(rettypes);
			trivia_0plus(p);
			var_name(p);
			trivia_0plus(p);
			p.advance(Syntax::Semicolon);
			p.close(member, Syntax::FieldDecl);
		}
		other => {
			p.cancel(rettypes);

			p.advance_err_and_close(
				member,
				Syntax::from(other),
				Syntax::Error,
				&[&[";", "`[`", "`,`", "`(`"]],
			);
		}
	}
}

/// Builds a [`Syntax::ParamList`] node. Includes delimiting parentheses.
fn param_list(p: &mut Parser<Syntax>) {
	p.debug_assert_at(Token::ParenL);
	let list = p.open();
	p.advance(Syntax::ParenL);
	trivia_0plus(p);

	if p.eat(Token::KwVoid, Syntax::KwVoid) {
		trivia_0plus(p);
		p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
		p.close(list, Syntax::ParamList);
		return;
	}

	while !p.at(Token::ParenR) && !p.eof() {
		parameter(p);
		trivia_0plus(p);

		if p.eat(Token::Comma, Syntax::Comma) {
			trivia_0plus(p);

			if p.at(Token::Dot3) {
				p.advance(Syntax::Dot3);
				break;
			} else if p.at(Token::ParenR) {
				p.advance_err_and_close(
					list,
					Syntax::from(p.nth(0)),
					Syntax::ParamList,
					&[&["a parameter"]],
				);
				return;
			}
		}
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syntax::ParenR, &[&["`)`"]]);
	p.close(list, Syntax::ParamList);
}

/// Builds a [`Syntax::Parameter`] node.
fn parameter(p: &mut Parser<Syntax>) {
	p.debug_assert_at_if(|t| !t.is_trivia());
	let param = p.open();

	loop {
		match p.nth(0) {
			Token::KwIn => p.advance(Syntax::KwIn),
			Token::KwOut => p.advance(Syntax::KwOut),
			_ => break,
		}

		trivia_0plus(p);
	}

	type_ref(p);
	trivia_0plus(p);
	ident_lax(p);

	if p.find(0, |token| !token.is_trivia()) == Token::Eq {
		trivia_0plus(p);
		p.advance(Syntax::Eq);
		trivia_0plus(p);
		expr(p);
	}

	p.close(param, Syntax::Parameter);
}

#[must_use]
fn in_decl_qual_first_set(token: Token) -> bool {
	matches!(
		token,
		Token::KwAbstract
			| Token::KwAction
			| Token::KwClearScope
			| Token::KwDeprecated
			| Token::KwFinal
			| Token::KwInternal
			| Token::KwMeta
			| Token::KwNative
			| Token::KwOverride
			| Token::KwPlay
			| Token::KwPrivate
			| Token::KwProtected
			| Token::KwReadOnly
			| Token::KwStatic
			| Token::KwTransient
			| Token::KwUi
			| Token::KwVarArg
			| Token::KwVersion
			| Token::KwVirtual
			| Token::KwVirtualScope
	)
}
