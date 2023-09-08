//! Parsers for definitions for classes, mixin classes, and structs.

use crate::{
	parser::Parser,
	zdoom::{zscript::Syn, Token},
};

use super::*;

/// Builds a [`Syn::ClassDef`] node.
pub fn class_def(p: &mut Parser<Syn>) {
	p.debug_assert_at_any(&[Token::KwClass, Token::DocComment]);
	let classdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwClass);
	p.advance(Syn::KwClass);
	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);

	if p.at(Token::Colon) {
		let inherit = p.open();
		p.advance(Syn::Colon);
		trivia_0plus(p);
		ident_lax(p);
		p.close(inherit, Syn::InheritSpec);
	}

	trivia_0plus(p);
	let quals = p.open();

	while !p.at(Token::BraceL) && !p.eof() {
		match p.nth(0) {
			Token::KwReplaces => {
				let replaces = p.open();
				p.advance(Syn::KwReplaces);
				trivia_0plus(p);
				ident_lax(p);
				p.close(replaces, Syn::ReplacesClause);
			}
			Token::KwAbstract => p.advance(Syn::KwAbstract),
			Token::KwPlay => p.advance(Syn::KwPlay),
			Token::KwUi => p.advance(Syn::KwUi),
			Token::KwNative => p.advance(Syn::KwNative),
			Token::KwVersion => version_qual(p),
			other => p.advance_with_error(
				Syn::from(other),
				&[
					"`abstract`",
					"`native`",
					"`play`",
					"`replaces`",
					"`ui`",
					"`version`",
				],
			),
		}

		trivia_0plus(p);
	}

	p.close(quals, Syn::ClassQuals);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		class_innard(p);
		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
	p.close(classdef, Syn::ClassDef);
}

/// Builds a [`Syn::MixinClassDef`] node.
pub fn mixin_class_def(p: &mut Parser<Syn>) {
	p.debug_assert_at_any(&[Token::KwMixin, Token::DocComment]);
	let mixindef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwMixin);
	p.advance(Syn::KwMixin);
	trivia_0plus(p);
	p.expect(Token::KwClass, Syn::KwClass, &["`class`"]);
	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		class_innard(p);
		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
	p.close(mixindef, Syn::MixinClassDef);
}

/// Builds a [`Syn::StructDef`] node.
pub fn struct_def(p: &mut Parser<Syn>) {
	p.debug_assert_at_any(&[Token::KwStruct, Token::DocComment]);
	let structdef = p.open();
	doc_comments(p);
	p.debug_assert_at(Token::KwStruct);
	p.advance(Syn::KwStruct);
	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);
	let quals = p.open();

	while !p.at(Token::BraceL) && !p.eof() {
		match p.nth(0) {
			Token::KwPlay => p.advance(Syn::KwPlay),
			Token::KwUi => p.advance(Syn::KwUi),
			Token::KwNative => p.advance(Syn::KwNative),
			Token::KwClearScope => p.advance(Syn::KwClearScope),
			Token::KwVersion => version_qual(p),
			other => p.advance_with_error(
				Syn::from(other),
				&["`clearscope`", "`native`", "`play`", "`ui`", "`version`"],
			),
		}

		trivia_0plus(p);
	}

	p.close(quals, Syn::StructQuals);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		struct_innard(p);
		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);

	if p.find(0, |token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syn::Semicolon);
	}

	p.close(structdef, Syn::StructDef);
}

/// Builds a [`Syn::ClassExtend`] or [`Syn::StructExtend`] node.
pub fn class_or_struct_extend(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwExtend);
	let extension = p.open();
	p.advance(Syn::KwExtend);
	trivia_0plus(p);

	let node_syn = match p.nth(0) {
		Token::KwClass => {
			p.advance(Syn::KwClass);
			Syn::ClassExtend
		}
		Token::KwStruct => {
			p.advance(Syn::KwStruct);
			Syn::StructExtend
		}
		other => {
			p.advance_err_and_close(
				extension,
				Syn::from(other),
				Syn::Unknown,
				&["`class`", "`struct`"],
			);
			return;
		}
	};

	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);
	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	if node_syn == Syn::ClassExtend {
		while !p.at(Token::BraceR) && !p.eof() {
			class_innard(p);
			trivia_0plus(p);
		}
	} else if node_syn == Syn::StructExtend {
		while !p.at(Token::BraceR) && !p.eof() {
			struct_innard(p);
			trivia_0plus(p);
		}
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);
	p.close(extension, node_syn);
}

// Innards /////////////////////////////////////////////////////////////////////

fn class_innard(p: &mut Parser<Syn>) {
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
		p.advance(Syn::Comment);
		return;
	}

	match token {
		Token::KwDefault => default_block(p),
		Token::KwStates => states_block(p),
		Token::KwMixin => {
			let mixin = p.open();
			p.advance(Syn::KwMixin);
			trivia_0plus(p);
			ident_lax(p);
			trivia_0plus(p);
			p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(mixin, Syn::MixinStat);
		}
		other => p.advance_with_error(
			Syn::from(other),
			&[
				"a type name",
				"`mixin`",
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
			],
		),
	}
}

fn struct_innard(p: &mut Parser<Syn>) {
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
			Syn::from(other),
			&[
				"a type name",
				"`const` or `enum`",
				"`play` or `ui` or `virtualscope` or `clearscope`",
				"`deprecated` or `version`",
				"`abstract` or `final` or `override` or `virtual`",
				"`private` or `protected`",
				"`internal` or `meta` or `native` or `transient`",
				"`static` or `readonly`",
				"`vararg`",
			],
		),
	}

	trivia_0plus(p);
}

/// Builds a [`Syn::FieldDecl`] or [`Syn::FunctionDecl`] node.
pub(super) fn member_decl(p: &mut Parser<Syn>) {
	let member = p.open();
	doc_comments(p);
	let quals = p.open();

	while p.at_if(in_decl_qual_first_set) && !p.eof() {
		match p.nth(0) {
			Token::KwDeprecated => deprecation_qual(p),
			Token::KwVersion => version_qual(p),
			Token::KwAction => {
				let action = p.open();
				p.advance(Syn::KwAction);

				if p.find(0, |token| !token.is_trivia()) == Token::ParenL {
					trivia_0plus(p);
					states_usage(p);
				}

				p.close(action, Syn::ActionQual);
			}
			Token::KwReadOnly => {
				if p.find(1, |token| !token.is_trivia()) == Token::AngleL {
					break; // We are likely looking at a `readonly<T>` return type.
				}

				p.advance(Syn::KwReadOnly);
			}
			other => p.advance(Syn::from(other)),
		}

		trivia_0plus(p);
	}

	p.close(quals, Syn::MemberQuals);
	let rettypes = p.open();
	type_ref(p);

	while p.find(0, |token| !token.is_trivia()) == Token::Comma {
		trivia_0plus(p);
		p.advance(Syn::Comma);
		trivia_0plus(p);
		type_ref(p);
	}

	trivia_0plus(p);

	if !p.at_if(is_ident_lax) {
		p.cancel(rettypes);
		p.advance_err_and_close(member, Syn::from(p.nth(0)), Syn::Error, &["an identifier"]);
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
					p.advance(Syn::Comma);
					trivia_0plus(p);
				} else {
					break;
				}
			}

			p.expect(Token::Semicolon, Syn::Semicolon, &["`;`"]);
			p.close(member, Syn::FieldDecl);
		}
		Token::ParenL => {
			p.close(rettypes, Syn::ReturnTypes);
			trivia_0plus(p);
			ident_lax(p);
			trivia_0plus(p);
			param_list(p);
			trivia_0plus(p);

			if p.eat(Token::KwConst, Syn::KwConst) {
				trivia_0plus(p);
			}

			if p.eat(Token::Semicolon, Syn::Semicolon) {
				p.close(member, Syn::FunctionDecl);
				return;
			}

			compound_stat(p);
			p.close(member, Syn::FunctionDecl);
		}
		Token::Semicolon => {
			p.cancel(rettypes);
			trivia_0plus(p);
			var_name(p);
			trivia_0plus(p);
			p.advance(Syn::Semicolon);
			p.close(member, Syn::FieldDecl);
		}
		other => {
			p.cancel(rettypes);

			p.advance_err_and_close(
				member,
				Syn::from(other),
				Syn::Error,
				&[";", "`[`", "`,`", "`(`"],
			);
		}
	}
}

/// Builds a [`Syn::ParamList`] node. Includes delimiting parentheses.
fn param_list(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::ParenL);
	let list = p.open();
	p.advance(Syn::ParenL);
	trivia_0plus(p);

	if p.eat(Token::KwVoid, Syn::KwVoid) {
		trivia_0plus(p);
		p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
		p.close(list, Syn::ParamList);
		return;
	}

	while !p.at(Token::ParenR) && !p.eof() {
		parameter(p);
		trivia_0plus(p);

		if p.eat(Token::Comma, Syn::Comma) {
			trivia_0plus(p);

			if p.at(Token::Dot3) {
				p.advance(Syn::Dot3);
				break;
			} else if p.at(Token::ParenR) {
				p.advance_err_and_close(
					list,
					Syn::from(p.nth(0)),
					Syn::ParamList,
					&["a parameter"],
				);
				return;
			}
		}
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
	p.close(list, Syn::ParamList);
}

/// Builds a [`Syn::Parameter`] node.
fn parameter(p: &mut Parser<Syn>) {
	p.debug_assert_at_if(|t| !t.is_trivia());
	let param = p.open();

	loop {
		match p.nth(0) {
			Token::KwIn => p.advance(Syn::KwIn),
			Token::KwOut => p.advance(Syn::KwOut),
			_ => break,
		}

		trivia_0plus(p);
	}

	type_ref(p);
	trivia_0plus(p);
	ident_lax(p);

	if p.find(0, |token| !token.is_trivia()) == Token::Eq {
		trivia_0plus(p);
		p.advance(Syn::Eq);
		trivia_0plus(p);
		expr(p);
	}

	p.close(param, Syn::Parameter);
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
