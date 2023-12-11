use crate::{
	parser::Parser,
	zdoom::{zscript::Syntax, Token},
};

use super::common::*;

/// Builds a [`Syntax::TypeRef`] node.
pub fn type_ref(p: &mut Parser<Syntax>) {
	let tref = p.open();
	core_type(p);

	while p.find(0, |token| !token.is_trivia()) == Token::BracketL {
		trivia_0plus(p);
		array_len(p);
	}

	p.close(tref, Syntax::TypeRef);
}

/// Builds a node tagged with one of the following:
/// - [`Syntax::ClassType`]
/// - [`Syntax::DynArrayType`]
/// - [`Syntax::IdentChainType`]
/// - [`Syntax::LetType`]
/// - [`Syntax::MapType`]
/// - [`Syntax::MapIterType`]
/// - [`Syntax::NativeType`]
/// - [`Syntax::ReadOnlyType`]
pub fn core_type(p: &mut Parser<Syntax>) {
	let cty = p.open();
	let token = p.nth(0);

	if is_ident::<0>(token) {
		ident_chain::<0>(p);
		p.close(cty, Syntax::IdentChainType);
		return;
	}

	if is_primitive_type(token) {
		p.advance(Syntax::from(token));
		p.close(cty, Syntax::PrimitiveType);
		return;
	}

	match token {
		Token::KwLet => {
			p.advance(Syntax::KwLet);
			p.close(cty, Syntax::LetType);
		}
		Token::KwArray => {
			p.advance(Syntax::KwArray);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syntax::AngleL, &[&["`<`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syntax::AngleR, &[&["`>`"]]);
			p.close(cty, Syntax::DynArrayType);
		}
		Token::KwClass => {
			p.advance(Syntax::KwClass);

			if p.find(0, |token| !token.is_trivia()) == Token::AngleL {
				trivia_0plus(p);
				p.advance(Syntax::AngleL);
				trivia_0plus(p);
				ident_chain::<{ ID_SFKW | ID_SQKW | ID_TYPES }>(p);
				trivia_0plus(p);
				p.expect(Token::AngleR, Syntax::AngleR, &[&["`>`"]]);
			}

			p.close(cty, Syntax::ClassType);
		}
		Token::KwMap => {
			p.advance(Syntax::KwMap);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syntax::AngleL, &[&["`<`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::Comma, Syntax::Comma, &[&["`,`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syntax::AngleR, &[&["`>`"]]);
			p.close(cty, Syntax::MapType);
		}
		Token::KwMapIterator => {
			p.advance(Syntax::KwMapIterator);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syntax::AngleL, &[&["`<`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::Comma, Syntax::Comma, &[&["`,`"]]);
			trivia_0plus(p);
			type_ref(p);
			trivia_0plus(p);
			p.expect(Token::AngleR, Syntax::AngleR, &[&["`>`"]]);
			p.close(cty, Syntax::MapIterType);
		}
		Token::KwReadOnly => {
			p.advance(Syntax::KwReadOnly);
			trivia_0plus(p);
			p.expect(Token::AngleL, Syntax::AngleL, &[&["`<`"]]);
			trivia_0plus(p);

			let t = p.nth(0);

			if is_ident::<0>(t) {
				ident::<0>(p);
			} else if t == Token::At {
				p.advance(Syntax::At);
				trivia_0plus(p);
				ident::<0>(p);
			} else {
				p.advance_err_and_close(
					cty,
					Syntax::from(t),
					Syntax::ReadOnlyType,
					&[&["an identifier", "`@`"]],
				);

				return;
			}

			trivia_0plus(p);
			p.expect(Token::AngleR, Syntax::AngleR, &[&["`>`"]]);
			p.close(cty, Syntax::ReadOnlyType);
		}
		Token::At => {
			p.advance(Syntax::At);
			trivia_0plus(p);
			ident::<0>(p);
			p.close(cty, Syntax::NativeType);
		}
		Token::Dot => {
			ident_chain::<0>(p);
			p.close(cty, Syntax::IdentChainType);
		}
		other => {
			p.advance_err_and_close(
				cty,
				Syntax::from(other),
				Syntax::Error,
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
