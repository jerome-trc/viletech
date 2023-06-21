//! Combinators applicable to multiple other parts of the syntax.

use chumsky::{primitive, IterParser, Parser};
use rowan::{GreenNode, GreenToken};

use crate::{
	comb, parser_t,
	parsing::*,
	zdoom::{
		zscript::{parse::expr, Syn},
		Token,
	},
	GreenElement,
};

use super::ParserBuilder;

impl ParserBuilder {
	/// The returned parser emits a [`Syn::ArrayLen`] node.
	pub(super) fn array_len<'i>(&self) -> parser_t!(Vec<GreenNode>) {
		primitive::group((
			comb::just_ts(Token::BracketL, Syn::BracketL),
			self.trivia_0plus(),
			self.expr().or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::BracketR, Syn::BracketR),
		))
		.map(|group| coalesce_node(group, Syn::ArrayLen))
		.repeated()
		.at_least(1)
		.collect()
	}

	/// The returned parser emits a [`Syn::DeprecationQual`] node.
	pub fn deprecation_qual<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwDeprecated, Syn::KwDeprecated),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenR),
			self.trivia_0plus(),
			comb::just_ts(Token::StringLit, Syn::StringLit),
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				comb::just_ts(Token::StringLit, Syn::StringLit),
			))
			.or_not(),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR),
		))
		.map(|group| coalesce_node(group, Syn::DeprecationQual))
	}

	/// The returned parser emits a [`Syn::Ident`] token.
	pub(super) fn ident<'i>(&self) -> parser_t!(GreenToken) {
		primitive::any()
			.filter(|token: &Token| {
				matches!(
					token,
					Token::Ident
						| Token::KwBright | Token::KwFast
						| Token::KwSlow | Token::KwNoDelay
						| Token::KwCanRaise | Token::KwOffset
						| Token::KwLight
				)
			})
			.map_with_state(comb::green_token(Syn::Ident))
	}

	/// The returned parser emits a [`Syn::IdentChain`] node.
	pub fn ident_chain<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			self.ident(),
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Dot, Syn::Dot),
				self.ident(),
			))
			.repeated()
			.collect::<Vec<_>>(),
		))
		.map(|group| coalesce_node(group, Syn::IdentChain))
	}

	/// The returned parser emits a series of [`Syn::Ident`] tokens (comma-separated).
	pub fn ident_list<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.ident().map(|gtok| vec![gtok.into()]).foldl(
			primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				self.ident(),
			))
			.repeated(),
			|mut lhs, (mut t0, comma, mut t1, ident)| {
				lhs.append(&mut t0);
				lhs.push(comma.into());
				lhs.append(&mut t1);
				lhs.push(ident.into());
				lhs
			},
		)
	}

	/// The returned parser emits a [`Syn::Whitespace`] or [`Syn::Comment`] token.
	pub(super) fn trivia<'i>(&self) -> parser_t!(GreenElement) {
		primitive::choice((
			comb::just_ts(Token::Whitespace, Syn::Whitespace),
			comb::just_ts(Token::Comment, Syn::Comment),
			comb::just_ts(Token::RegionStart, Syn::RegionStart),
			comb::just_ts(Token::RegionEnd, Syn::RegionEnd),
		))
		.map(|token| token.into())
	}

	/// Shorthand for `self.trivia().repeated().collect()`.
	pub(super) fn trivia_0plus<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.trivia().repeated().collect()
	}

	/// Shorthand for `self.trivia().repeated().at_least(1).collect()`.
	pub(super) fn trivia_1plus<'i>(&self) -> parser_t!(Vec<GreenElement>) {
		self.trivia().repeated().at_least(1).collect()
	}

	/// The returned parser emits a [`Syn::TypeRef`] node.
	pub fn type_ref<'i>(&self) -> parser_t!(GreenNode) {
		chumsky::recursive::recursive(|tref| {
			let at_ident = primitive::group((comb::just_ts(Token::At, Syn::At), self.ident()))
				.map(|group| coalesce_node(group, Syn::NativeType));

			let ident = self
				.ident()
				.map(|gtok| GreenNode::new(Syn::IdentChain.into(), [gtok.into()]));

			let readonly = primitive::group((
				comb::just_ts(Token::KwReadonly, Syn::KwReadonly),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				primitive::choice((ident, at_ident.clone())),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::ReadonlyType));

			let tref_identchain = self
				.ident_chain()
				.map(|gnode| GreenNode::new(Syn::IdentChainType.into(), [gnode.into()]));

			let tref_let = comb::just_ts(Token::KwLet, Syn::KwLet)
				.map(|gtok| GreenNode::new(Syn::LetType.into(), [gtok.into()]));

			let simple = primitive::choice((readonly, at_ident, tref_identchain, tref_let));

			let tref_or_fixedlen_array =
				primitive::group((tref.clone(), self.array_len().or_not())).map(coalesce_vec);

			let class_restrictor = primitive::group((
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				self.ident_chain(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(coalesce_vec);

			let tref_class = primitive::group((
				comb::just_ts(Token::KwClass, Syn::KwClass),
				class_restrictor.or_not(),
			))
			.map(|group| coalesce_node(group, Syn::ClassType));

			let tref_array_dyn = primitive::group((
				comb::just_ts(Token::KwArray, Syn::KwArray),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::DynArrayType));

			let tref_map = primitive::group((
				comb::just_ts(Token::KwMap, Syn::KwMap),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::MapType));

			let tref_mapiter = primitive::group((
				comb::just_ts(Token::KwMapIterator, Syn::KwMapIterator),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleL, Syn::AngleL),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::Comma, Syn::Comma),
				self.trivia_0plus(),
				tref_or_fixedlen_array.clone(),
				self.trivia_0plus(),
				comb::just_ts(Token::AngleR, Syn::AngleR),
			))
			.map(|group| coalesce_node(group, Syn::MapIterType));

			primitive::choice((tref_class, tref_array_dyn, tref_map, tref_mapiter, simple)).boxed()
		})
	}

	/// The returned parser emits a [`Syn::VarName`] node.
	pub fn var_name<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			self.ident(),
			primitive::group((self.trivia_0plus(), self.array_len())).or_not(),
		))
		.map(|group| coalesce_node(group, Syn::VarName))
	}

	/// The returned parser emits a [`Syn::VersionQual`] node.
	pub fn version_qual<'i>(&self) -> parser_t!(GreenNode) {
		primitive::group((
			comb::just_ts(Token::KwVersion, Syn::KwVersion),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenL, Syn::ParenL),
			self.trivia_0plus(),
			comb::just_ts(Token::StringLit, Syn::StringLit),
			self.trivia_0plus(),
			comb::just_ts(Token::ParenR, Syn::ParenR),
		))
		.map(|group| coalesce_node(group, Syn::VersionQual))
	}
}

/// Builds a [`Syn::ArrayLen`] node.
pub(super) fn array_len(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at(Token::BracketL));
	let l = p.open();
	p.advance(Syn::BracketL);
	trivia_0plus(p);

	if p.at_if(expr::in_first_set) {
		expr(p);
	}

	trivia_0plus(p);
	p.expect(Token::BracketR, Syn::BracketR, &["`]`"]);
	p.close(l, Syn::ArrayLen);
}

/// Builds a [`Syn::DeprecationQual`] node.
pub(super) fn deprecation_qual(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at(Token::KwDeprecated));
	let qual = p.open();
	p.advance(Syn::KwDeprecated);
	trivia_0plus(p);
	p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syn::StringLit, &["a version string"]);
	trivia_0plus(p);

	if p.eat(Token::Comma, Syn::Comma) {
		trivia_0plus(p);
		p.expect(Token::StringLit, Syn::StringLit, &["a reason string"]);
	}

	trivia_0plus(p);
	p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
	p.close(qual, Syn::DeprecationQual);
}

pub(super) fn ident(p: &mut crate::parser::Parser<Syn>) {
	p.expect_any(IDENT_TOKENS, &["an identifier"])
}

#[must_use]
pub(super) fn is_ident(token: Token) -> bool {
	IDENT_TOKENS.iter().any(|t| t.0 == token)
}

#[must_use]
pub(super) fn eat_ident(p: &mut crate::parser::Parser<Syn>) -> bool {
	p.eat_any(IDENT_TOKENS)
}

/// Like [`ident`] but allows [`Token::KwProperty`] and builtin type names.
pub(super) fn ident_lax(p: &mut crate::parser::Parser<Syn>) {
	p.expect_if(is_ident_lax, Syn::Ident, &["an identifier"])
}

#[must_use]
pub(super) fn is_ident_lax(token: Token) -> bool {
	is_ident(token) || IDENT_TOKENS_LAX.iter().any(|t| t.0 == token)
}

const IDENT_TOKENS: &[(Token, Syn)] = &[
	(Token::Ident, Syn::Ident),
	(Token::KwBright, Syn::Ident),
	(Token::KwCanRaise, Syn::Ident),
	(Token::KwFast, Syn::Ident),
	(Token::KwLight, Syn::Ident),
	(Token::KwOffset, Syn::Ident),
	(Token::KwSlow, Syn::Ident),
];

const IDENT_TOKENS_LAX: &[(Token, Syn)] = &[
	(Token::KwInt16, Syn::Ident),
	(Token::KwSByte, Syn::Ident),
	(Token::KwByte, Syn::Ident),
	(Token::KwInt8, Syn::Ident),
	(Token::KwUInt8, Syn::Ident),
	(Token::KwShort, Syn::Ident),
	(Token::KwUShort, Syn::Ident),
	(Token::KwInt16, Syn::Ident),
	(Token::KwUInt16, Syn::Ident),
	(Token::KwInt, Syn::Ident),
	(Token::KwUInt, Syn::Ident),
	(Token::KwFloat, Syn::Ident),
	(Token::KwDouble, Syn::Ident),
	(Token::KwString, Syn::Ident),
	(Token::KwVector2, Syn::Ident),
	(Token::KwVector3, Syn::Ident),
	// Curiously, ZScript's Lemon grammar prescribes a `vector4` keyword as
	// being an option here, but there's no RE2C lexer rule for it.
	(Token::KwName, Syn::Ident),
	(Token::KwMap, Syn::Ident),
	(Token::KwMapIterator, Syn::Ident),
	(Token::KwArray, Syn::Ident),
	(Token::KwVoid, Syn::Ident),
	(Token::KwState, Syn::Ident),
	(Token::KwColor, Syn::Ident),
	(Token::KwSound, Syn::Ident),
	(Token::KwProperty, Syn::Ident),
];

/// Builds a [`Syn::IdentChain`] node. Also see [`ident_chain_lax`].
pub fn ident_chain(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at_if(is_ident));
	let chain = p.open();
	p.advance(Syn::Ident);

	while p.next_filtered(|token| !token.is_trivia()) == Token::Dot {
		trivia_0plus(p);
		p.advance(Syn::Dot);
		trivia_0plus(p);
		ident(p);
	}

	p.close(chain, Syn::IdentChain);
}

/// Like [`ident_chain`] but backed by [`is_ident_lax`].
pub fn ident_chain_lax(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at_if(is_ident_lax));
	let chain = p.open();
	p.advance(Syn::Ident);

	while p.next_filtered(|token| !token.is_trivia()) == Token::Dot {
		trivia_0plus(p);
		p.advance(Syn::Dot);
		trivia_0plus(p);
		ident_lax(p);
	}

	p.close(chain, Syn::IdentChain);
}

/// Builds a series of [`Syn::Ident`] tokens, separated by trivia and commas.
/// Returns `true` if more than one identifier was parsed.
pub fn ident_list(p: &mut crate::parser::Parser<Syn>) -> bool {
	let mut ret = false;
	ident(p);

	while p.next_filtered(|token| !token.is_trivia()) == Token::Comma {
		trivia_0plus(p);
		p.advance(Syn::Comma);
		trivia_0plus(p);
		ident(p);
		ret = true;
	}

	ret
}

/// May or may not build a token tagged with one of the following:
/// - [`Syn::Whitespace`]
/// - [`Syn::Comment`]
/// - [`Syn::RegionStart`]
/// - [`Syn::RegionEnd`]
pub(super) fn trivia(p: &mut crate::parser::Parser<Syn>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syn::Whitespace),
		(Token::Comment, Syn::Comment),
		(Token::RegionStart, Syn::RegionStart),
		(Token::RegionEnd, Syn::RegionEnd),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut crate::parser::Parser<Syn>) {
	while trivia(p) {}
}

/// Expects one [`trivia`] and then calls [`trivia_0plus`].
pub(super) fn trivia_1plus(p: &mut crate::parser::Parser<Syn>) {
	p.expect_any(
		&[
			(Token::Whitespace, Syn::Whitespace),
			(Token::Comment, Syn::Comment),
			(Token::RegionStart, Syn::RegionStart),
			(Token::RegionEnd, Syn::RegionEnd),
		],
		&["whitespace or a comment (one or more)"],
	);

	trivia_0plus(p);
}

/// Builds a node tagged with one of the following:
/// - [`Syn::ClassType`]
/// - [`Syn::DynArrayType`]
/// - [`Syn::IdentChainType`]
/// - [`Syn::MapIterType`]
/// - [`Syn::MapType`]
/// - [`Syn::ReadonlyType`]
/// - [`Syn::NativeType`]
/// - [`Syn::LetType`]
pub fn type_ref(p: &mut crate::parser::Parser<Syn>) {
	fn tref_with_optional_arraylen(p: &mut crate::parser::Parser<Syn>) {
		type_ref(p);

		if p.next_filtered(|token| !token.is_trivia()) == Token::BracketL {
			trivia_0plus(p);
			p.advance(Syn::BracketL);
			trivia_0plus(p);
			expr(p);
			trivia_0plus(p);
			p.expect(Token::BracketR, Syn::BracketR, &["`]`"]);
		}
	}

	let tref = p.open();

	let token = p.nth(0);

	if is_ident(token) {
		ident_chain(p);
		p.close(tref, Syn::IdentChainType);
		return;
	}

	if is_primitive_type(token) {
		p.advance(Syn::from(token));
		p.close(tref, Syn::PrimitiveType);
		return;
	}

	match token {
		Token::KwLet => {
			p.advance(Syn::KwLet);
			p.close(tref, Syn::LetType);
		}
		Token::KwArray => {
			p.advance(Syn::KwArray);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
			trivia_0plus(p);
			tref_with_optional_arraylen(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			p.close(tref, Syn::DynArrayType);
		}
		Token::KwClass => {
			p.advance(Syn::KwClass);

			if p.next_filtered(|token| !token.is_trivia()) == Token::AngleL {
				trivia_0plus(p);
				p.advance(Syn::AngleL);
				trivia_0plus(p);
				ident_chain(p);
				trivia_0plus(p);
				p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			}

			p.close(tref, Syn::ClassType);
		}
		Token::KwMap => {
			p.advance(Syn::KwMap);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
			trivia_0plus(p);
			tref_with_optional_arraylen(p);
			trivia_0plus(p);
			p.expect(Token::Comma, Syn::Comma, &["`,`"]);
			trivia_0plus(p);
			tref_with_optional_arraylen(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			p.close(tref, Syn::MapType);
		}
		Token::KwMapIterator => {
			p.advance(Syn::KwMapIterator);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
			trivia_0plus(p);
			tref_with_optional_arraylen(p);
			trivia_0plus(p);
			p.expect(Token::Comma, Syn::Comma, &["`,`"]);
			trivia_0plus(p);
			tref_with_optional_arraylen(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			p.close(tref, Syn::MapIterType);
		}
		Token::KwReadonly => {
			p.advance(Syn::KwReadonly);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
			trivia_0plus(p);

			let t = p.nth(0);

			if is_ident(t) {
				ident(p);
			} else if t == Token::At {
				p.advance(Syn::At);
				ident(p);
			} else {
				p.advance_err_and_close(
					tref,
					Syn::from(t),
					Syn::ReadonlyType,
					&["an identifier", "`@`"],
				);
				return;
			}

			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			p.close(tref, Syn::ReadonlyType);
		}
		Token::At => {
			p.advance(Syn::At);
			trivia_0plus(p);
			ident(p);
			p.close(tref, Syn::NativeType);
		}
		other => {
			p.advance_err_and_close(
				tref,
				Syn::from(other),
				Syn::Error,
				&[
					"`let`",
					"`class`",
					"`array`",
					"`map`",
					"`mapiterator`",
					"`readonly`",
					"`@`",
					"an identifier",
				],
			);
		}
	}
}

/// i.e. can `token` begin a [type reference](type_ref)?
/// Note that this includes (non-lax) identifiers.
#[must_use]
pub(super) fn in_type_ref_first_set(token: Token) -> bool {
	if is_ident(token) || is_primitive_type(token) {
		return true;
	}

	matches!(
		token,
		Token::KwLet | Token::KwArray | Token::KwMap | Token::KwMapIterator | Token::KwReadonly
	)
}

#[must_use]
fn is_primitive_type(token: Token) -> bool {
	matches!(
		token,
		Token::KwSByte
			| Token::KwByte
			| Token::KwInt8
			| Token::KwUInt8
			| Token::KwShort
			| Token::KwUShort
			| Token::KwInt16
			| Token::KwUInt16
			| Token::KwInt
			| Token::KwUInt
			| Token::KwBool
			| Token::KwFloat
			| Token::KwDouble
			| Token::KwVector2
			| Token::KwVector3
			| Token::KwName
			| Token::KwSound
			| Token::KwState
			| Token::KwColor
	)
}

/// Builds a [`Syn::VarName`] node.
pub(super) fn var_name(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at_if(is_ident));
	let name = p.open();
	p.advance(Syn::Ident);

	if p.next_filtered(|token| !token.is_trivia()) == Token::BracketL {
		trivia_0plus(p);
		array_len(p);
	}

	p.close(name, Syn::VarName);
}

/// Builds a [`Syn::VersionQual`] node.
pub(super) fn version_qual(p: &mut crate::parser::Parser<Syn>) {
	debug_assert!(p.at(Token::KwVersion));
	let qual = p.open();
	p.advance(Syn::KwVersion);
	trivia_0plus(p);
	p.expect(Token::ParenL, Syn::ParenL, &["`(`"]);
	trivia_0plus(p);
	p.expect(Token::StringLit, Syn::StringLit, &["a version string"]);
	trivia_0plus(p);
	p.expect(Token::ParenR, Syn::ParenR, &["`)`"]);
	p.close(qual, Syn::VersionQual);
}

#[cfg(test)]
mod test {
	use crate::{
		testing::*,
		zdoom::{self, zscript::ParseTree},
	};

	use super::*;

	#[test]
	fn smoke_identlist() {
		const SOURCE: &str = r#"temple, of, the, ancient, techlords"#;

		let ptree: ParseTree = crate::parse(
			SOURCE,
			|p| {
				ident_list(p);
			},
			zdoom::Version::default(),
		);
		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_typeref() {
		const SOURCES: &[&str] = &[
			"TeenyLittleBase",
			"Dead.On.Arrival",
			"readonly<Corruption2Factory>",
			"class",
			"class<Forge>",
			"array<Unwelcome>",
			"array<class<TheOssuary> >",
			"map<Corruption[1], Mortem[2][3]>",
			"mapiterator<FishInABarrel, Neoplasm>",
		];

		for source in SOURCES {
			let ptree: ParseTree = crate::parse(source, type_ref, zdoom::Version::default());
			assert_no_errors(&ptree);
		}
	}

	#[test]
	fn smoke_version_qual() {
		const SOURCE: &str = r#"version("3.7.1")"#;

		let ptree: ParseTree = crate::parse(SOURCE, version_qual, zdoom::Version::default());
		assert_no_errors(&ptree);
	}

	#[test]
	fn smoke_deprecation_qual() {
		const SOURCE: &str = r#"deprecated("2.4.0", "Don't use this please")"#;

		let ptree: ParseTree = crate::parse(SOURCE, deprecation_qual, zdoom::Version::default());
		assert_no_errors(&ptree);
	}
}
