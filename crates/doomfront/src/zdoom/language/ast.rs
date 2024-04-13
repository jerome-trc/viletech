//! Abstract syntax tree nodes.

use crate::simple_astnode;

use super::{Syntax, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syntax::KeyValuePair`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct KeyValuePair(SyntaxNode);

simple_astnode!(Syntax, KeyValuePair, Syntax::KeyValuePair);

impl KeyValuePair {
	/// The returned token is tagged [`Syntax::Ident`].
	#[must_use]
	pub fn key(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.unwrap()
	}

	#[must_use]
	pub fn game_qualifier(&self) -> Option<GameQualifier> {
		self.0
			.first_child()
			.filter(|node| node.kind() == Syntax::GameQualifier)
			.map(GameQualifier)
	}

	/// Returns all tokens under this node tagged [`Syntax::StringLit`].
	pub fn string_parts(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|elem| {
			elem.into_token()
				.filter(|token| token.kind() == Syntax::StringLit)
		})
	}
}

/// Wraps a node tagged [`Syntax::GameQualifier`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GameQualifier(SyntaxNode);

simple_astnode!(Syntax, GameQualifier, Syntax::GameQualifier);

impl GameQualifier {
	/// The returned token is tagged [`Syntax::Ident`].
	#[must_use]
	pub fn game_id(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.unwrap()
	}
}

/// Wraps a node tagged [`Syntax::Header`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Header(SyntaxNode);

simple_astnode!(Syntax, Header, Syntax::Header);

impl Header {
	/// Yielded tokens will be tagged with one of the following:
	/// - [`Syntax::Ident`]
	/// - [`Syntax::KwDefault`]
	/// - [`Syntax::Asterisk`]
	/// - [`Syntax::Tilde`]
	pub fn contents(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|elem| {
			let Some(token) = elem.into_token() else {
				return None;
			};

			match token.kind() {
				Syntax::Ident | Syntax::KwDefault | Syntax::Asterisk | Syntax::Tilde => Some(token),
				_ => None,
			}
		})
	}
}
