//! Abstract syntax tree nodes.

mod actor;
mod expr;
mod lit;

use rowan::{ast::AstNode, NodeOrToken};

use crate::simple_astnode;

use super::{syntax::Syntax, SyntaxNode, SyntaxToken};

pub use self::{actor::*, expr::*, lit::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TopLevel {
	ActorDef(ActorDef),
	ConstDef(ConstDef),
	EnumDef(EnumDef),
	IncludeDirective(IncludeDirective),
}

impl AstNode for TopLevel {
	type Language = Syntax;

	fn can_cast(kind: <Self::Language as rowan::Language>::Kind) -> bool
	where
		Self: Sized,
	{
		matches!(
			kind,
			Syntax::ActorDef | Syntax::ConstDef | Syntax::EnumDef | Syntax::IncludeDirective
		)
	}

	fn cast(node: rowan::SyntaxNode<Self::Language>) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::ActorDef => Some(Self::ActorDef(ActorDef(node))),
			Syntax::ConstDef => Some(Self::ConstDef(ConstDef(node))),
			Syntax::EnumDef => Some(Self::EnumDef(EnumDef(node))),
			Syntax::IncludeDirective => Some(Self::IncludeDirective(IncludeDirective(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::ActorDef(inner) => inner.syntax(),
			Self::ConstDef(inner) => inner.syntax(),
			Self::EnumDef(inner) => inner.syntax(),
			Self::IncludeDirective(inner) => inner.syntax(),
		}
	}
}

impl TopLevel {
	#[must_use]
	pub fn into_actordef(self) -> Option<ActorDef> {
		match self {
			Self::ActorDef(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_constdef(self) -> Option<ConstDef> {
		match self {
			Self::ConstDef(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_enumdef(self) -> Option<EnumDef> {
		match self {
			Self::EnumDef(inner) => Some(inner),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_include_directive(self) -> Option<IncludeDirective> {
		match self {
			Self::IncludeDirective(inner) => Some(inner),
			_ => None,
		}
	}
}

// ConstDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ConstDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConstDef(SyntaxNode);

simple_astnode!(Syntax, ConstDef, Syntax::ConstDef);

impl ConstDef {
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| (elem.kind() == Syntax::Ident).then(|| elem.into_token().unwrap()))
			.unwrap()
	}

	#[must_use]
	pub fn type_spec(&self) -> ConstType {
		self.syntax()
			.children_with_tokens()
			.find_map(|elem| {
				let NodeOrToken::Token(tok) = elem else {
					return None;
				};

				if tok.text().eq_ignore_ascii_case("int") {
					Some(ConstType::Int)
				} else if tok.text().eq_ignore_ascii_case("float") {
					Some(ConstType::Float)
				} else {
					None
				}
			})
			.unwrap()
	}

	#[must_use]
	pub fn expr(&self) -> Expr {
		Expr::cast(self.syntax().last_child().unwrap()).unwrap()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConstType {
	Int,
	Float,
}

// EnumDef /////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::EnumDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EnumDef(SyntaxNode);

simple_astnode!(Syntax, EnumDef, Syntax::EnumDef);

impl EnumDef {
	pub fn variants(&self) -> impl Iterator<Item = EnumVariant> {
		self.syntax().children().filter_map(EnumVariant::cast)
	}
}

/// Wraps a node tagged [`Syntax::EnumVariant`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EnumVariant(SyntaxNode);

simple_astnode!(Syntax, EnumVariant, Syntax::EnumVariant);

impl EnumVariant {
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.syntax().first_token().unwrap()
	}

	#[must_use]
	pub fn initializer(&self) -> Option<Expr> {
		self.syntax()
			.last_child()
			.map(|node| Expr::cast(node).unwrap())
	}
}

// IncludeDirective ////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::IncludeDirective`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IncludeDirective(SyntaxNode);

simple_astnode!(Syntax, IncludeDirective, Syntax::IncludeDirective);

impl IncludeDirective {
	#[must_use]
	pub fn path(&self) -> SyntaxToken {
		self.syntax().last_token().unwrap()
	}
}

// IdentChain //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::IdentChain`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IdentChain(SyntaxNode);

simple_astnode!(Syntax, IdentChain, Syntax::IdentChain);

impl IdentChain {
	/// Each yielded token is tagged [`Syntax::Ident`].
	pub fn parts(&self) -> impl Iterator<Item = SyntaxToken> {
		self.syntax()
			.children_with_tokens()
			.filter_map(|elem| elem.into_token().filter(|tok| tok.kind() == Syntax::Ident))
	}
}
