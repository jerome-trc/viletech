use crate::{
	parser::Parser,
	zdoom::{zscript::Syn, Token},
};

use super::common::*;

/// Builds a [`Syn::TypeRef`] node.
pub fn type_ref(p: &mut Parser<Syn>) {
	let tref = p.open();
	core_type(p);

	while p.find(0, |token| !token.is_trivia()) == Token::BracketL {
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
			p.expect(Token::AngleL, Syn::AngleL, &[&["`<`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &[&["`>`"]]);
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
				p.expect(Token::AngleR, Syn::AngleR, &[&["`>`"]]);
			}

			p.close(cty, Syn::ClassType);
		}
		Token::KwMap => {
			p.advance(Syn::KwMap);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &[&["`<`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::Comma, Syn::Comma, &[&["`,`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &[&["`>`"]]);
			p.close(cty, Syn::MapType);
		}
		Token::KwMapIterator => {
			p.advance(Syn::KwMapIterator);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &[&["`<`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::Comma, Syn::Comma, &[&["`,`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &[&["`>`"]]);
			p.close(cty, Syn::MapIterType);
		}
		Token::KwReadOnly => {
			p.advance(Syn::KwReadOnly);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syn::AngleL, &[&["`<`"]]);
			trivia_0plus(p);

			let t = p.nth(0);

			if is_ident::<0>(t) {
				ident::<0>(p);
			} else if t == Token::At {
				p.advance(Syn::At);
				trivia_0plus(p);
				ident::<0>(p);
			} else {
				p.advance_err_and_close(
					cty,
					Syn::from(t),
					Syn::ReadOnlyType,
					&[&["an identifier", "`@`"]],
				);

				return;
			}

			trivia_0plus(p);
			p.expect(Token::AngleR, Syn::AngleR, &[&["`>`"]]);
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
				&[&[
					"`let`",
					"`class`",
					"`array`",
					"`map`",
					"`mapiterator`",
					"`readonly`",
					"`@`",
					"`.`",
					"an identifier",
				]],
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
