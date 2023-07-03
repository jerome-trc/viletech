//! Combinators applicable to multiple other parts of the syntax.

use crate::{
	parser::Parser,
	zdoom::{zscript::Syn, Token},
};

use super::expr;

// Identifiers /////////////////////////////////////////////////////////////////

/// Allows the following to be considered identifiers:
/// - [`Token::KwLoop`]
/// - [`Token::KwFail`]
/// - [`Token::KwWait`]
/// - [`Token::KwOffset`]
/// - [`Token::KwSlow`]
pub(super) const ID_SFKW: u8 = 1 << 0;

/// Allows the following to be considered identifiers:
/// - [`Token::KwBright`]
/// - [`Token::KwCanRaise`]
/// - [`Token::KwFast`]
/// - [`Token::KwLight`]
/// - [`Token::KwOffset`]
/// - [`Token::KwSlow`]
pub(super) const ID_SQKW: u8 = 1 << 1;

/// Allows [`Token::KwProperty`] and builtin type names.
pub(super) const ID_TYPES: u8 = 1 << 2;

/// Allows [`Token::KwDefault`].
pub(super) const ID_DEFAULT: u8 = 1 << 3;

const STATEFLOW_KWS: &[Token] = &[
	Token::KwLoop,
	Token::KwFail,
	Token::KwWait,
	Token::KwOffset,
	Token::KwSlow,
];

const STATEQUAL_KWS: &[Token] = &[
	Token::KwBright,
	Token::KwCanRaise,
	Token::KwFast,
	Token::KwLight,
	Token::KwOffset,
	Token::KwSlow,
];

const PRIMTYPE_KWS: &[Token] = &[
	Token::KwInt16,
	Token::KwSByte,
	Token::KwByte,
	Token::KwInt8,
	Token::KwUInt8,
	Token::KwShort,
	Token::KwUShort,
	Token::KwInt16,
	Token::KwUInt16,
	Token::KwInt,
	Token::KwUInt,
	Token::KwFloat,
	Token::KwDouble,
	Token::KwString,
	Token::KwVector2,
	Token::KwVector3,
	// Curiously, ZScript's Lemon grammar prescribes a `vector4` keyword as
	// being an option here, but there's no RE2C lexer rule for it.
	Token::KwName,
	Token::KwMap,
	Token::KwMapIterator,
	Token::KwArray,
	Token::KwVoid,
	Token::KwState,
	Token::KwColor,
	Token::KwSound,
	Token::KwProperty,
];

/// Combine [`ID_SFKW`], [`ID_SQKW`], and [`ID_TYPES`] via bitwise or to form `CFG`.
/// If `0` is given, only [`Token::Ident`] will match.
pub(super) fn ident<const CFG: u8>(p: &mut Parser<Syn>) {
	let token = p.nth(0);

	if is_ident::<CFG>(token) {
		p.advance(Syn::Ident);
	} else {
		p.advance_with_error(Syn::from(token), &["an identifier"])
	}
}

/// Combine [`ID_SFKW`], [`ID_SQKW`], and [`ID_TYPES`] via bitwise or to form `CFG`.
/// If `0` is given, only [`Token::Ident`] will match.
pub(super) fn is_ident<const CFG: u8>(token: Token) -> bool {
	if token == Token::Ident {
		return true;
	}

	if (CFG & ID_SFKW) != 0 && STATEFLOW_KWS.contains(&token) {
		return true;
	}

	if (CFG & ID_SQKW) != 0 && STATEQUAL_KWS.contains(&token) {
		return true;
	}

	if (CFG & ID_TYPES) != 0 && PRIMTYPE_KWS.contains(&token) {
		return true;
	}

	if (CFG & ID_DEFAULT) != 0 && token == Token::KwDefault {
		return true;
	}

	false
}

/// Shorthand for `ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);`.
pub(super) fn ident_lax(p: &mut Parser<Syn>) {
	ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
}

/// Shorthand for `is_ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(token);`.
#[must_use]
pub(super) fn is_ident_lax(token: Token) -> bool {
	is_ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(token)
}

/// Builds a [`Syn::IdentChain`] node.
/// Backed by [`is_ident`]; see that function's documentation for details on `CFG`.
pub(super) fn ident_chain<const CFG: u8>(p: &mut Parser<Syn>) {
	p.debug_assert_at_if(|token| is_ident::<CFG>(token) || token == Token::Dot);

	let chain = p.open();

	if p.eat(Token::Dot, Syn::Dot) {
		trivia_0plus(p);
	}

	p.advance(Syn::Ident);

	while p.find(0, |token| !token.is_trivia()) == Token::Dot {
		trivia_0plus(p);
		p.advance(Syn::Dot);
		trivia_0plus(p);
		ident::<CFG>(p);
	}

	p.close(chain, Syn::IdentChain);
}

/// Builds a series of [`Syn::Ident`] tokens, separated by trivia and commas.
/// Returns `true` if more than one identifier was parsed.
/// Backed by [`is_ident`]; see that function's documentation for details on `CFG`.
pub(super) fn ident_list<const CFG: u8>(p: &mut Parser<Syn>) -> bool {
	let mut ret = false;
	ident::<CFG>(p);

	while p.find(0, |token| !token.is_trivia()) == Token::Comma {
		trivia_0plus(p);
		p.advance(Syn::Comma);
		trivia_0plus(p);
		ident::<CFG>(p);
		ret = true;
	}

	ret
}

// Types ///////////////////////////////////////////////////////////////////////

/// Builds a [`Syn::TypeRef`] node.
pub fn type_ref(p: &mut Parser<Syn>) {
	let tref = p.open();
	core_type(p);

	if p.find(0, |token| !token.is_trivia()) == Token::BracketL {
		trivia_0plus(p);
		array_len(p);
	}

	p.close(tref, Syn::TypeRef);
}

/// Builds a node tagged with one of the following:
/// - [`Syn::ClassType`]
/// - [`Syn::DynArrayType`]
/// - [`Syn::IdentChainType`]
/// - [`Syn::LetType`]
/// - [`Syn::MapType`]
/// - [`Syn::MapIterType`]
/// - [`Syn::NativeType`]
/// - [`Syn::ReadOnlyType`]
pub fn core_type(p: &mut Parser<Syn>) {
	let cty = p.open();
	let token = p.nth(0);

	if is_ident::<0>(token) {
		ident_chain::<0>(p);
		p.close(cty, Syn::IdentChainType);
		return;
	}

	if is_primitive_type(token) {
		p.advance(Syn::from(token));
		p.close(cty, Syn::PrimitiveType);
		return;
	}

	match token {
		Token::KwLet => {
			p.advance(Syn::KwLet);
			p.close(cty, Syn::LetType);
		}
		Token::KwArray => {
			p.advance(Syn::KwArray);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			p.close(cty, Syn::DynArrayType);
		}
		Token::KwClass => {
			p.advance(Syn::KwClass);

			if p.find(0, |token| !token.is_trivia()) == Token::AngleL {
				trivia_0plus(p);
				p.advance(Syn::AngleL);
				trivia_0plus(p);
				ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
				trivia_0plus(p);
				p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			}

			p.close(cty, Syn::ClassType);
		}
		Token::KwMap => {
			p.advance(Syn::KwMap);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::Comma, Syn::Comma, &["`,`"]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			p.close(cty, Syn::MapType);
		}
		Token::KwMapIterator => {
			p.advance(Syn::KwMapIterator);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::Comma, Syn::Comma, &["`,`"]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			p.close(cty, Syn::MapIterType);
		}
		Token::KwReadOnly => {
			p.advance(Syn::KwReadOnly);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &["`<`"]);
			trivia_0plus(p);

			let t = p.nth(0);

			if is_ident::<0>(t) || t == Token::At {
				ident::<0>(p);
			} else if t == Token::At {
				p.advance(Syn::At);
				ident::<0>(p);
			} else {
				p.advance_err_and_close(
					cty,
					Syn::from(t),
					Syn::ReadOnlyType,
					&["an identifier", "`@`"],
				);
				return;
			}

			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &["`>`"]);
			p.close(cty, Syn::ReadOnlyType);
		}
		Token::At => {
			p.advance(Syn::At);
			trivia_0plus(p);
			ident::<0>(p);
			p.close(cty, Syn::NativeType);
		}
		Token::Dot => {
			ident_chain::<0>(p);
			p.close(cty, Syn::IdentChainType);
		}
		other => {
			p.advance_err_and_close(
				cty,
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
					"`.`",
					"an identifier",
				],
			);
		}
	}
}

/// i.e. can `token` begin a [type reference](type_ref)?
/// Note that this includes (non-lax) identifiers and [`Token::KwLet`].
#[must_use]
pub(super) fn in_type_ref_first_set(token: Token) -> bool {
	if is_ident::<0>(token) || is_primitive_type(token) {
		return true;
	}

	matches!(
		token,
		Token::KwLet
			| Token::KwClass
			| Token::KwArray
			| Token::KwMap
			| Token::KwMapIterator
			| Token::KwReadOnly
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
			| Token::KwString
			| Token::KwColor
			| Token::KwVoid
	)
}

// Miscellaneous ///////////////////////////////////////////////////////////////

/// Builds a [`Syn::ArrayLen`] node.
pub(super) fn array_len(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::BracketL);
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
pub(super) fn deprecation_qual(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwDeprecated);
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

/// Parse 0 or more [`Token::DocComment`]s, additionally consuming trailing trivia.
pub(super) fn doc_comments(p: &mut Parser<Syn>) {
	while p.eat(Token::DocComment, Syn::DocComment) {
		trivia_0plus(p);
	}
}

/// May or may not build a token tagged with one of the following:
/// - [`Syn::Whitespace`]
/// - [`Syn::Comment`]
/// - [`Syn::RegionStart`]
/// - [`Syn::RegionEnd`]
pub(super) fn trivia(p: &mut Parser<Syn>) -> bool {
	p.eat_any(&[
		(Token::Whitespace, Syn::Whitespace),
		(Token::Comment, Syn::Comment),
		(Token::RegionStart, Syn::RegionStart),
		(Token::RegionEnd, Syn::RegionEnd),
	])
}

/// Shorthand for `while trivia(p) {}`.
pub(super) fn trivia_0plus(p: &mut Parser<Syn>) {
	while trivia(p) {}
}

/// Expects one [`trivia`] and then calls [`trivia_0plus`].
pub(super) fn trivia_1plus(p: &mut Parser<Syn>) {
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

/// Builds a [`Syn::VarName`] node.
pub(super) fn var_name(p: &mut Parser<Syn>) {
	p.debug_assert_at_if(is_ident::<{ ID_SFKW | ID_SQKW | ID_TYPES }>);
	let name = p.open();
	p.advance(Syn::Ident);

	loop {
		if p.find(0, |token| !token.is_trivia()) == Token::BracketL {
			trivia_0plus(p);
			array_len(p);
		} else {
			break;
		}
	}

	p.close(name, Syn::VarName);
}

/// Builds a [`Syn::VersionQual`] node.
pub(super) fn version_qual(p: &mut Parser<Syn>) {
	p.debug_assert_at(Token::KwVersion);
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
	use rowan::ast::AstNode;

	use crate::{
		testing::*,
		zdoom::{
			self,
			zscript::{ast, parse, ParseTree},
		},
	};

	use super::*;

	#[test]
	fn smoke_identlist() {
		const SOURCE: &str = r#"property temple: of, the, ancient, techlords;"#;

		let ptree: ParseTree = crate::parse(
			SOURCE,
			parse::property_def,
			zdoom::lex::Context::ZSCRIPT_LATEST,
		);

		assert_no_errors(&ptree);
		prettyprint_maybe(ptree.cursor());
	}

	#[test]
	fn smoke_types() {
		const SOURCES: &[&str] = &[
			"TeenyLittleBase",
			"Dead.On.Arrival",
			"readonly<Corruption2Factory>",
			"class",
			"class<Forge>",
			"array<Unwelcome>",
			"array<class<TheOssuary> >",
			"map<Corruption[1], Mortem[2]>",
			"mapiterator<FishInABarrel, Neoplasm>",
		];

		for source in SOURCES {
			let ptree: ParseTree =
				crate::parse(source, type_ref, zdoom::lex::Context::ZSCRIPT_LATEST);
			assert_no_errors(&ptree);
			prettyprint_maybe(ptree.cursor());
		}
	}

	#[test]
	fn smoke_version_qual() {
		const SOURCE: &str = r#"version("3.7.1")"#;

		let ptree: ParseTree =
			crate::parse(SOURCE, version_qual, zdoom::lex::Context::ZSCRIPT_LATEST);
		assert_no_errors(&ptree);
		let qual = ast::VersionQual::cast(ptree.cursor()).unwrap();
		assert_eq!(qual.version().unwrap().string().unwrap(), "3.7.1");
	}

	#[test]
	fn smoke_deprecation_qual() {
		const SOURCE: &str = r#"deprecated("2.4.0", "Don't use this please")"#;

		let ptree: ParseTree = crate::parse(
			SOURCE,
			deprecation_qual,
			zdoom::lex::Context::ZSCRIPT_LATEST,
		);

		assert_no_errors(&ptree);
		let qual = ast::DeprecationQual::cast(ptree.cursor()).unwrap();
		assert_eq!(qual.version().unwrap().string().unwrap(), "2.4.0");
		assert_eq!(qual.message().unwrap().text(), "\"Don't use this please\"");
	}
}
