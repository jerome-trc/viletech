//! Abstract syntax tree nodes.

use rowan::ast::AstNode;

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
		self.0.first_token().unwrap()
	}

	/// Returns all tokens under this node tagged [`Syn::StringLit`].
	pub fn string_parts(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|elem| {
			elem.into_token()
				.filter(|token| token.kind() == Syn::StringLit)
		})
	}
}

/// Wraps a node tagged [`Syn::LocaleTag`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LocaleTag(SyntaxNode);

impl LocaleTag {
	/// The returned token is tagged [`Syn::Ident`] and its text will be, for
	/// example, "enu" or "eng".
	pub fn locale(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// If the [locale identifier](Self::locale) is followed by the keyword
	/// `default`, this returns `true`.
	#[must_use]
	pub fn is_default(&self) -> bool {
		self.0
			.children_with_tokens()
			.any(|elem| elem.kind() == Syn::KwDefault)
	}
}
