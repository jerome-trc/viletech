//! Abstract syntax tree nodes.

use rowan::ast::AstNode;

use crate::{simple_astnode, zdoom::ast::LitToken};

use super::{Syntax, SyntaxNode, SyntaxToken};

/// Wraps a node tagged [`Syntax::Definition`].
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
	/// - [`Syntax::KwInt`]
	/// - [`Syntax::KwFloat`]
	/// - [`Syntax::KwBool`]
	/// - [`Syntax::KwColor`]
	/// - [`Syntax::KwString`]
	/// - [`Syntax::Error`]
	#[must_use]
	pub fn type_spec(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| match elem.kind() {
				Syntax::KwInt
				| Syntax::KwFloat
				| Syntax::KwBool
				| Syntax::KwString
				| Syntax::KwColor
				| Syntax::Error => elem.into_token(),
				_ => None,
			})
			.unwrap()
	}

	/// The identifier given to this CVar, after the type specifier.
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.unwrap()
	}

	#[must_use]
	pub fn default(&self) -> Option<DefaultDef> {
		self.0.children().find_map(DefaultDef::cast)
	}
}

simple_astnode!(Syntax, Definition, Syntax::Definition);

/// Wraps a node tagged [`Syntax::DefFlags`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DefFlags(SyntaxNode);

impl DefFlags {
	/// The kind of the returned token will be one of the following:
	/// - [`Syntax::KwServer`]
	/// - [`Syntax::KwUser`]
	/// - [`Syntax::KwNoSave`]
	#[must_use]
	pub fn scope(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token().filter(|token| {
					matches!(
						token.kind(),
						Syntax::KwServer | Syntax::KwUser | Syntax::KwNoSave
					)
				})
			})
			.unwrap()
	}

	/// The kinds of the yielded tokens (if any) will each be one of the following:
	/// - [`Syntax::KwNoArchive`]
	/// - [`Syntax::KwCheat`]
	/// - [`Syntax::KwLatch`]
	pub fn qualifiers(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0.children_with_tokens().filter_map(|elem| {
			elem.into_token().filter(|token| {
				matches!(
					token.kind(),
					Syntax::KwNoArchive | Syntax::KwCheat | Syntax::KwLatch
				)
			})
		})
	}
}

simple_astnode!(Syntax, DefFlags, Syntax::DefFlags);

/// Wraps a node tagged [`Syntax::DefaultDef`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DefaultDef(SyntaxNode);

simple_astnode!(Syntax, DefaultDef, Syntax::DefaultDef);

impl DefaultDef {
	#[must_use]
	pub fn token(&self) -> LitToken<Syntax> {
		LitToken::new(self.0.last_token().unwrap())
	}
}
