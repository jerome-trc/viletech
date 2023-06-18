//! AST nodes for parts of definitions for classes inheriting from `Actor`.

use rowan::ast::AstNode;

use crate::{simple_astnode, zdoom::decorate::ast::StateUsage};

use super::{IdentChain, Syn, SyntaxNode, SyntaxToken};

// FlagDef /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FlagDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlagDef(SyntaxNode);

simple_astnode!(Syn, FlagDef, Syn::FlagDef);

impl FlagDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

// PropertyDef /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PropertyDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropertyDef(SyntaxNode);

simple_astnode!(Syn, PropertyDef, Syn::PropertyDef);

impl PropertyDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

// DefaultBlock ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::DefaultBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DefaultBlock(pub(super) SyntaxNode);

simple_astnode!(Syn, DefaultBlock, Syn::DefaultBlock);

// DefaultItem /////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DefaultItem {
	FlagSetting(FlagSetting),
	PropertySetting(PropertySetting),
}

impl AstNode for DefaultItem {
	type Language = Syn;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::FlagSetting | Syn::PropertySetting)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::FlagSetting => Some(Self::FlagSetting(FlagSetting(node))),
			Syn::PropertySetting => Some(Self::PropertySetting(PropertySetting(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			DefaultItem::FlagSetting(inner) => inner.syntax(),
			DefaultItem::PropertySetting(inner) => inner.syntax(),
		}
	}
}

// FlagSetting /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FlagSetting`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FlagSetting(pub(super) SyntaxNode);

simple_astnode!(Syn, FlagSetting, Syn::FlagSetting);

impl FlagSetting {
	#[must_use]
	pub fn is_adding(&self) -> bool {
		self.0.first_token().unwrap().kind() == Syn::Plus
	}

	#[must_use]
	pub fn is_removing(&self) -> bool {
		self.0.first_token().unwrap().kind() == Syn::Minus
	}

	#[must_use]
	pub fn name(&self) -> IdentChain {
		IdentChain::cast(self.syntax().last_child().unwrap()).unwrap()
	}
}

// PropertySetting /////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::PropertySetting`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PropertySetting(pub(super) SyntaxNode);

simple_astnode!(Syn, PropertySetting, Syn::PropertySetting);

impl PropertySetting {
	#[must_use]
	pub fn name(&self) -> IdentChain {
		IdentChain::cast(self.0.first_child().unwrap()).unwrap()
	}
}

// StatesBlock /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StatesBlock`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StatesBlock(pub(super) SyntaxNode);

simple_astnode!(Syn, StatesBlock, Syn::StatesBlock);

impl StatesBlock {
	#[must_use]
	pub fn usage_quals(&self) -> Option<impl Iterator<Item = StateUsage>> {
		self.syntax()
			.first_child()
			.filter(|node| node.kind() == Syn::StatesUsage)
			.map(|node| {
				node.children_with_tokens().filter_map(|elem| {
					let Some(token) = elem.into_token() else { return None; };

					if token.text().eq_ignore_ascii_case("actor") {
						Some(StateUsage::Actor)
					} else if token.text().eq_ignore_ascii_case("item") {
						Some(StateUsage::Item)
					} else if token.text().eq_ignore_ascii_case("overlay") {
						Some(StateUsage::Overlay)
					} else if token.text().eq_ignore_ascii_case("weapon") {
						Some(StateUsage::Weapon)
					} else {
						None
					}
				})
			})
	}
}
