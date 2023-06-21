//! Parsers for definitions for classes, mixin classes, and structs.

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::*;

impl ParserBuilder {
	/// The returned parser emits a [`Syn::ClassDef`] node.
	pub fn class_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwClass, Syn::KwClass),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			self.inherit_spec().or_not(),
			self.trivia_0plus(),
			primitive::group((self.class_qual(), self.trivia_1plus()))
				.repeated()
				.collect::<Vec<_>>(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.class_innard().repeated().collect::<Vec<_>>(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::ClassDef))
		.boxed()
	}

	/// The returned parser emits a [`Syn::InheritSpec`] node.
	fn inherit_spec<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::Colon, Syn::Colon),
			self.trivia_0plus(),
			self.ident(),
		))
		.map(|group| coalesce_node(group, Syn::InheritSpec))
	}

	fn class_qual<'i>(&self) -> parser_t!(GreenElement) {
		let replaces = primitive::group((
			self.trivia_1plus(),
			comb::just_ts(Token::KwReplaces, Syn::KwReplaces),
			self.trivia_1plus(),
			self.ident(),
		))
		.map(|group| coalesce_node(group, Syn::ReplacesClause));

		primitive::choice((
			comb::just_ts(Token::KwAbstract, Syn::KwAbstract).map(GreenElement::from),
			comb::just_ts(Token::KwNative, Syn::KwNative).map(GreenElement::from),
			comb::just_ts(Token::KwPlay, Syn::KwPlay).map(GreenElement::from),
			comb::just_ts(Token::KwUi, Syn::KwUi).map(GreenElement::from),
			self.version_qual().map(GreenElement::from),
			replaces.map(GreenElement::from),
		))
	}

	/// The returned parser emits a [`Syn::ClassExtend`] node.
	pub fn class_extend<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwExtend, Syn::KwExtend),
			self.trivia_1plus(),
			comb::just_ts(Token::KwClass, Syn::KwClass),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.class_innard().repeated().collect::<Vec<_>>(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::ClassExtend))
		.boxed()
	}

	/// The returned parser emits a [`Syn::StructDef`] node.
	pub fn struct_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwStruct, Syn::KwStruct),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			primitive::group((
				primitive::choice((
					comb::just_ts(Token::KwClearScope, Syn::KwClearScope).map(GreenElement::from),
					comb::just_ts(Token::KwNative, Syn::KwNative).map(GreenElement::from),
					comb::just_ts(Token::KwPlay, Syn::KwPlay).map(GreenElement::from),
					comb::just_ts(Token::KwUi, Syn::KwUi).map(GreenElement::from),
					self.version_qual().map(GreenElement::from),
				)),
				self.trivia_1plus(),
			))
			.repeated()
			.collect::<Vec<_>>(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.struct_innard().repeated().collect::<Vec<_>>(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::StructDef))
		.boxed()
	}

	/// The returned parser emits a [`Syn::StructExtend`] node.
	pub fn struct_extend<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwExtend, Syn::KwExtend),
			self.trivia_1plus(),
			comb::just_ts(Token::KwStruct, Syn::KwStruct),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.struct_innard().repeated().collect::<Vec<_>>(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::StructExtend))
		.boxed()
	}

	/// The returned parser emits a [`Syn::MixinClassDef`] node.
	pub fn mixin_class_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwMixin, Syn::KwMixin),
			self.trivia_1plus(),
			comb::just_ts(Token::KwClass, Syn::KwClass),
			self.trivia_1plus(),
			self.ident(),
			self.trivia_0plus(),
			comb::just_ts(Token::BraceL, Syn::BraceL),
			self.class_innard().repeated().collect::<Vec<_>>(),
			comb::just_ts(Token::BraceR, Syn::BraceR),
		))
		.map(|group| coalesce_node(group, Syn::MixinClassDef))
		.boxed()
	}

	// Innards /////////////////////////////////////////////////////////////////

	fn class_innard<'i>(&self) -> parser_t!(GreenElement) {
		primitive::choice((
			self.trivia(),
			self.func_decl().map(GreenElement::from),
			self.field_decl().map(GreenElement::from),
			self.const_def().map(GreenElement::from),
			self.enum_def().map(GreenElement::from),
			self.states_block().map(GreenElement::from),
			self.default_block().map(GreenElement::from),
			self.property_def().map(GreenElement::from),
			self.flag_def().map(GreenElement::from),
		))
	}

	fn struct_innard<'i>(&self) -> parser_t!(GreenElement) {
		primitive::choice((
			self.trivia(),
			self.enum_def().map(GreenElement::from),
			self.const_def().map(GreenElement::from),
			self.func_decl().map(GreenElement::from),
			self.field_decl().map(GreenElement::from),
		))
	}

	fn decl_qual<'i>(&self) -> parser_t!(GreenElement) {
		let parameterized = primitive::choice((self.deprecation_qual(), self.version_qual()))
			.map(GreenElement::from);

		let kw = primitive::choice((
			comb::just_ts(Token::KwAbstract, Syn::KwAbstract),
			comb::just_ts(Token::KwClearScope, Syn::KwClearScope),
			comb::just_ts(Token::KwFinal, Syn::KwFinal),
			comb::just_ts(Token::KwInternal, Syn::KwInternal),
			// Curiously, the ZScript grammar prescribes a `latent` keyword as
			// being an option here, but there's no RE2C lexer rule for it.
			comb::just_ts(Token::KwMeta, Syn::KwMeta),
			comb::just_ts(Token::KwNative, Syn::KwNative),
			comb::just_ts(Token::KwOverride, Syn::KwOverride),
			comb::just_ts(Token::KwPlay, Syn::KwPlay),
			comb::just_ts(Token::KwPrivate, Syn::KwPrivate),
			comb::just_ts(Token::KwProtected, Syn::KwProtected),
			comb::just_ts(Token::KwReadonly, Syn::KwReadonly),
			comb::just_ts(Token::KwStatic, Syn::KwStatic),
			comb::just_ts(Token::KwTransient, Syn::KwTransient),
			comb::just_ts(Token::KwUi, Syn::KwUi),
			comb::just_ts(Token::KwVarArg, Syn::KwVarArg),
			comb::just_ts(Token::KwVirtual, Syn::KwVirtual),
			comb::just_ts(Token::KwVirtualScope, Syn::KwVirtualScope),
		))
		.map(GreenElement::from);

		primitive::choice((parameterized, kw))
	}

	/// The returned parser emits a [`Syn::FieldDecl`] node.
	fn field_decl<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			primitive::group((self.decl_qual(), self.trivia_1plus()))
				.repeated()
				.collect::<Vec<_>>(),
			self.trivia_1plus(),
			self.type_ref(),
			self.trivia_0plus(),
			self.ident_list(),
			comb::just_ts(Token::Semicolon, Syn::Semicolon),
		))
		.map(|group| coalesce_node(group, Syn::FieldDecl))
	}

	/// The returned parser emits a [`Syn::FunctionDecl`] node.
	fn func_decl<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			primitive::group((self.decl_qual(), self.trivia_1plus()))
				.repeated()
				.collect::<Vec<_>>(),
			self.ident(),
			self.trivia_0plus(),
			primitive::choice((
				comb::just_ts(Token::Semicolon, Syn::Semicolon).map(GreenElement::from),
				self.compound_stat(self.statement()).map(GreenElement::from),
			)),
		))
		.map(|group| coalesce_node(group, Syn::FunctionDecl))
	}
}

/// Builds a [`Syn::ClassDef`] node.
pub fn class_def(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwClass);
	let classdef = p.open();
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
pub fn mixin_class_def(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwMixin);
	let mixindef = p.open();
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
pub fn struct_def(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwStruct);
	let structdef = p.open();
	p.advance(Syn::KwStruct);
	trivia_0plus(p);
	ident_lax(p);
	trivia_0plus(p);

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

	p.expect(Token::BraceL, Syn::BraceL, &["`{`"]);
	trivia_0plus(p);

	while !p.at(Token::BraceR) && !p.eof() {
		struct_innard(p);
		trivia_0plus(p);
	}

	trivia_0plus(p);
	p.expect(Token::BraceR, Syn::BraceR, &["`}`"]);

	if p.next_filtered(|token| !token.is_trivia()) == Token::Semicolon {
		trivia_0plus(p);
		p.advance(Syn::Semicolon);
	}

	p.close(structdef, Syn::StructDef);
}

/// Builds a [`Syn::ClassExtend`] or [`Syn::StructExtend`] node.
pub fn class_or_struct_extend(p: &mut crate::parser::Parser<Syn>) {
	p.debug_assert_at(Token::KwExtend);
	let extension = p.open();
	p.advance(Syn::KwExtend);
	trivia_0plus(p);

	let node_syn = match p.nth(0) {
		Token::KwClass => Syn::ClassExtend,
		Token::KwStruct => Syn::StructExtend,
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

fn class_innard(p: &mut crate::parser::Parser<Syn>) {
	let token = p.nth(0);

	if token == Token::KwStatic
		&& p.lookahead_filtered(|token| !token.is_trivia()) == Token::KwConst
	{
		static_const_stat(p);
		return;
	}

	if in_type_ref_first_set(token) || in_decl_qual_first_set(token) {
		member_decl(p);
		trivia_0plus(p);
		return;
	}

	match token {
		Token::KwDefault => default_block(p),
		Token::KwStates => states_block(p),
		Token::KwConst => const_def(p),
		Token::KwEnum => enum_def(p),
		Token::KwProperty => property_def(p),
		Token::KwFlagDef => flag_def(p),
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
				"`const` or `enum`",
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

	trivia_0plus(p);
}

fn struct_innard(p: &mut crate::parser::Parser<Syn>) {
	let token = p.nth(0);

	if token == Token::KwStatic
		&& p.lookahead_filtered(|token| !token.is_trivia()) == Token::KwConst
	{
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
fn member_decl(p: &mut crate::parser::Parser<Syn>) {
	let member = p.open();

	while p.at_if(in_decl_qual_first_set) && !p.eof() {
		match p.nth(0) {
			Token::KwDeprecated => deprecation_qual(p),
			Token::KwVersion => version_qual(p),
			Token::KwAction => {
				let action = p.open();
				p.advance(Syn::KwAction);

				if p.next_filtered(|token| !token.is_trivia()) == Token::ParenL {
					trivia_0plus(p);
					states_usage(p);
				}

				p.close(action, Syn::ActionQual);
			}
			other => p.advance(Syn::from(other)),
		}

		trivia_0plus(p);
	}

	type_ref(p);
	trivia_0plus(p);

	if !p.at_if(is_ident_lax) {
		p.advance_err_and_close(member, Syn::from(p.nth(0)), Syn::Error, &["an identifier"]);
		return;
	}

	let peeked = p.next_filtered(|token| !token.is_trivia() && !is_ident_lax(token));

	match peeked {
		Token::BracketL | Token::Comma => {
			trivia_0plus(p);

			while !p.at(Token::Semicolon) && !p.eof() {
				var_name(p);

				if p.next_filtered(|token| !token.is_trivia()) == Token::Comma {
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
			trivia_0plus(p);
			ident_lax(p);
			trivia_0plus(p);
			p.advance(Syn::Semicolon);
			p.close(member, Syn::FieldDecl);
		}
		other => {
			p.advance_err_and_close(member, Syn::from(other), Syn::Error, &["`[`", "`,`", "`(`"]);
		}
	}
}

/// Builds a [`Syn::ParamList`] node. Includes delimiting parentheses.
fn param_list(p: &mut crate::parser::Parser<Syn>) {
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

			if p.at(Token::ParenR) {
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

	p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
	p.close(list, Syn::ParamList);
}

/// Builds a [`Syn::Parameter`] node.
fn parameter(p: &mut crate::parser::Parser<Syn>) {
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

	if p.next_filtered(|token| !token.is_trivia()) == Token::Eq {
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
			| Token::KwReadonly
			| Token::KwStatic
			| Token::KwTransient
			| Token::KwUi
			| Token::KwVarArg
			| Token::KwVersion
			| Token::KwVirtual
			| Token::KwVirtualScope
	)
}

#[cfg(test)]
mod test {
	use super::*;

	use crate::{testing::*, zdoom::zscript::ParseTree};

	#[test]
	fn smoke() {
		const SOURCE: &str = r#####"

class Rocketpack_Flare : Actor
{
	Default
	{
		RenderStyle "Add";
		Scale 0.25;
		Alpha 0.95;
		+NOGRAVITY
		+NOINTERACTION
		+THRUGHOST
		+DONTSPLASH
		+NOTIMEFREEZE
	}

	States
	{
		Spawn:
			FLER A 1 Bright NoDelay {
				A_FadeOut(0.3);
				A_SetScale(Scale.X - FRandom(0.005, 0.0075));
				Return A_JumpIf(Scale.X <= 0.0, "Null");
			}
			Loop;
	}
}

"#####;

		let ptree: ParseTree = crate::parse(SOURCE, file, zdoom::Version::default());
		assert_no_errors(&ptree);
	}
}
