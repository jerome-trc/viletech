//! Abstract syntax tree nodes.

use crate::simple_astnode;

use super::{Syn, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syn::KeyValuePair`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct KeyValuePair(SyntaxNode);

simple_astnode!(Syn, KeyValuePair, Syn::KeyValuePair);

impl KeyValuePair {
	/// The returned token is tagged [`Syn::Ident`].
	#[must_use]
	pub fn key(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	#[must_use]
	pub fn game_qualifier(&self) -> Option<GameQualifier> {
		self.0
			.first_child()
			.filter(|node| node.kind() == Syn::GameQualifier)
			.map(GameQualifier)
	}

	/// Returns all tokens under this node tagged [`Syn::StringLit`].
	pub fn string_parts(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|elem| {
			elem.into_token()
				.filter(|token| token.kind() == Syn::StringLit)
		})
	}
}

/// Wraps a node tagged [`Syn::GameQualifier`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GameQualifier(SyntaxNode);

simple_astnode!(Syn, GameQualifier, Syn::GameQualifier);

impl GameQualifier {
	/// The returned token is tagged [`Syn::Ident`].
	#[must_use]
	pub fn game_id(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

/// Wraps a node tagged [`Syn::Header`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Header(SyntaxNode);

simple_astnode!(Syn, Header, Syn::Header);

impl Header {
	/// Yielded tokens will be tagged with one of the following:
	/// - [`Syn::Ident`]
	/// - [`Syn::KwDefault`]
	/// - [`Syn::Asterisk`]
	/// - [`Syn::Tilde`]
	pub fn contents(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|elem| {
			let Some(token) = elem.into_token() else {
				return None;
			};

			match token.kind() {
				Syn::Ident | Syn::KwDefault | Syn::Asterisk | Syn::Tilde => Some(token),
				_ => None,
			}
		})
	}
}
