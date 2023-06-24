//! Abstract syntax tree nodes.

use rowan::ast::AstNode;

use crate::{simple_astnode, zdoom::ast::LitToken};

use super::{Syn, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syn::Definition`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Definition(SyntaxNode);

impl Definition {
	/// Everything preceding the storage type specifier.
	#[must_use]
	pub fn flags(&self) -> DefFlags {
		self.0
			.first_child()
			.unwrap()
			.children()
			.find_map(DefFlags::cast)
			.unwrap()
	}

	/// The storage type specifier follows the flags and scope specifier, and
	/// precededes the identifier.
	///
	/// Its kind will be one of the following:
	/// - [`Syn::KwInt`]
	/// - [`Syn::KwFloat`]
	/// - [`Syn::KwBool`]
	/// - [`Syn::KwColor`]
	/// - [`Syn::KwString`]
	/// - [`Syn::Error`]
	#[must_use]
	pub fn type_spec(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| match elem.kind() {
				Syn::KwInt
				| Syn::KwFloat
				| Syn::KwBool
				| Syn::KwString
				| Syn::KwColor
				| Syn::Error => elem.into_token(),
				_ => None,
			})
			.unwrap()
	}

	/// The identifier given to this CVar, after the type specifier.
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	#[must_use]
	pub fn default(&self) -> Option<DefaultDef> {
		self.0.children().find_map(DefaultDef::cast)
	}
}

simple_astnode!(Syn, Definition, Syn::Definition);

/// Wraps a node tagged [`Syn::DefFlags`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DefFlags(SyntaxNode);

impl DefFlags {
	/// The kind of the returned token will be one of the following:
	/// - [`Syn::KwServer`]
	/// - [`Syn::KwUser`]
	/// - [`Syn::KwNoSave`]
	#[must_use]
	pub fn scope(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token().filter(|token| {
					matches!(token.kind(), Syn::KwServer | Syn::KwUser | Syn::KwNoSave)
				})
			})
			.unwrap()
	}

	/// The kinds of the yielded tokens (if any) will each be one of the following:
	/// - [`Syn::KwNoArchive`]
	/// - [`Syn::KwCheat`]
	/// - [`Syn::KwLatch`]
	pub fn qualifiers(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|elem| {
			elem.into_token().filter(|token| {
				matches!(token.kind(), Syn::KwNoArchive | Syn::KwCheat | Syn::KwLatch)
			})
		})
	}
}

simple_astnode!(Syn, DefFlags, Syn::DefFlags);

/// Wraps a node tagged [`Syn::DefaultDef`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DefaultDef(SyntaxNode);

simple_astnode!(Syn, DefaultDef, Syn::DefaultDef);

impl DefaultDef {
	#[must_use]
	pub fn token(&self) -> LitToken<Syn> {
		LitToken::new(self.0.last_token().unwrap())
	}
}
