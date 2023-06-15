//! Parsers for definitions for classes, mixin classes, and structs.

use chumsky::{primitive, IterParser, Parser};
use rowan::GreenNode;

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{zscript::Syn, Token},
	GreenElement,
};

use super::ParserBuilder;

impl ParserBuilder {
	/// The returned parser emits a [`Syn::ClassDef`] node.
	pub fn class_def<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwClass, Syn::KwClass),
			self.trivia_1plus(),
			self.ident(),
			self.inherit_spec().or_not(),
			primitive::group((self.class_qual(), self.trivia_1plus()))
				.repeated()
				.collect::<Vec<_>>(),
			self.trivia_1plus(),
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
			self.trivia_1plus(),
			comb::just_ts(Token::KwReplaces, Syn::KwReplaces),
			self.trivia_1plus(),
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
	}

	// Innards /////////////////////////////////////////////////////////////////

	fn class_innard<'i>(&self) -> parser_t!(GreenElement) {
		primitive::choice((
			self.trivia(),
			self.func_decl().map(GreenElement::from),
			self.field_decl().map(GreenElement::from),
			self.const_def().map(GreenElement::from),
			self.enum_def().map(GreenElement::from),
			self.states_def().map(GreenElement::from),
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
