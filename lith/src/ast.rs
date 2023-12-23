//! Structures for representing Lithica abstract syntax trees.

mod expr;
mod lit;
mod pat;
mod stmt;

use doomfront::{
	rowan::{ast::AstNode, Direction, TextRange},
	simple_astnode, AstError, AstResult,
};

use crate::{Syntax, SyntaxNode, SyntaxToken};

pub use self::{expr::*, lit::*, pat::*, stmt::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TopLevel {
	Annotation(Annotation),
	Statement(Statement),
}

impl AstNode for TopLevel {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		Statement::can_cast(kind) || matches!(kind, Syntax::Annotation)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		if let Some(stmt) = Statement::cast(node.clone()) {
			return Some(Self::Statement(stmt));
		}

		if let Some(anno) = Annotation::cast(node) {
			return Some(Self::Annotation(anno));
		}

		None
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Annotation(inner) => inner.syntax(),
			Self::Statement(inner) => inner.syntax(),
		}
	}
}

// Annotation //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::Annotation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Annotation(pub(super) SyntaxNode);

simple_astnode!(Syntax, Annotation, Syntax::Annotation);

impl Annotation {
	pub fn name(&self) -> AstResult<AnnotationName> {
		let ident0 = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syntax::Ident))
			.ok_or(AstError::Missing)?;

		let dot_opt = ident0
			.siblings_with_tokens(Direction::Next)
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syntax::Dot));

		let Some(dot) = dot_opt else {
			return Ok(AnnotationName::Unscoped(ident0));
		};

		if let Some(ident1) = dot
			.siblings_with_tokens(Direction::Next)
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syntax::Ident))
		{
			return Ok(AnnotationName::Scoped(ident0, ident1));
		}

		Ok(AnnotationName::Unscoped(ident0))
	}

	#[must_use]
	pub fn arg_list(&self) -> Option<ArgList> {
		self.0.last_child().and_then(ArgList::cast)
	}
}

/// All tokens herein are always tagged [`Syntax::Ident`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AnnotationName {
	Unscoped(SyntaxToken),
	Scoped(SyntaxToken, SyntaxToken),
}

impl AnnotationName {
	#[must_use]
	pub fn text(&self) -> (&str, Option<&str>) {
		match self {
			Self::Unscoped(ident) => (ident.text(), None),
			Self::Scoped(ident0, ident1) => (ident0.text(), Some(ident1.text())),
		}
	}

	#[must_use]
	pub fn text_range(&self) -> TextRange {
		match self {
			Self::Unscoped(ident) => ident.text_range(),
			Self::Scoped(ident0, ident1) => {
				TextRange::new(ident0.text_range().start(), ident1.text_range().end())
			}
		}
	}
}

// ArgList, Argument ///////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ArgList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ArgList(SyntaxNode);

simple_astnode!(Syntax, ArgList, Syntax::ArgList);

impl ArgList {
	/// The returned token is always tagged [`Syntax::Dot3`].
	#[must_use]
	pub fn dot3(&self) -> Option<SyntaxToken> {
		let Some(ret) = self
			.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syntax::Dot3))
		else {
			return None;
		};

		if ret
			.next_sibling_or_token()
			.is_some_and(|elem| matches!(elem.kind(), Syntax::Argument | Syntax::Error))
		{
			return None;
		}

		Some(ret)
	}

	/// Shorthand for `self.dot3.is_some()`.
	#[must_use]
	pub fn acceding(&self) -> bool {
		self.dot3().is_some()
	}

	pub fn iter(&self) -> impl Iterator<Item = Argument> {
		self.0.children().filter_map(Argument::cast)
	}
}

/// Wraps a node tagged [`Syntax::Argument`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Argument(SyntaxNode);

simple_astnode!(Syntax, Argument, Syntax::Argument);

impl Argument {
	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	#[must_use]
	pub fn name(&self) -> Option<Name> {
		let mut name = None;
		let mut colon_seen = false;

		for elem in self.0.children_with_tokens() {
			match elem.kind() {
				Syntax::Colon => colon_seen = true,
				Syntax::Ident | Syntax::LitName => {
					name = Some(elem.into_token().unwrap());
				}
				_ => {}
			}
		}

		if colon_seen {
			name.map(Name)
		} else {
			None
		}
	}
}

// BlockLabel //////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::BlockLabel`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BlockLabel(SyntaxNode);

simple_astnode!(Syntax, BlockLabel, Syntax::BlockLabel);

impl BlockLabel {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn ident(&self) -> AstResult<SyntaxToken> {
		let Some(opener) = self.0.first_token() else {
			return Err(AstError::Incorrect);
		};

		let Some(closer) = self.0.last_token() else {
			return Err(AstError::Incorrect);
		};

		if opener.kind() != Syntax::Colon2 || closer.kind() != Syntax::Colon2 {
			return Err(AstError::Incorrect);
		}

		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syntax::Ident))
			.ok_or(AstError::Missing)
	}
}

// DocComment //////////////////////////////////////////////////////////////////

/// Wraps a [`Syntax::DocComment`] token. Provides a convenience function for
/// stripping preceding slashes and surrounding whitespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocComment(SyntaxToken);

impl DocComment {
	#[must_use]
	pub fn text_trimmed(&self) -> &str {
		self.0.text().trim_matches('/').trim()
	}
}

impl std::ops::Deref for DocComment {
	type Target = SyntaxToken;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

// Name ////////////////////////////////////////////////////////////////////////

/// A convenience for positions which accept either [`Syntax::Ident`] or [`Syntax::LitName`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Name(pub(super) SyntaxToken);

impl Name {
	#[must_use]
	pub fn text(&self) -> &str {
		match self.0.kind() {
			Syntax::Ident => self.0.text(),
			Syntax::LitName => &self.0.text()[1..(self.0.text().len() - 1)],
			_ => unreachable!(),
		}
	}

	#[must_use]
	pub fn inner(&self) -> &SyntaxToken {
		&self.0
	}
}

// TypeSpec ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::TypeSpec`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypeSpec(SyntaxNode);

simple_astnode!(Syntax, TypeSpec, Syntax::TypeSpec);

impl TypeSpec {
	#[must_use]
	pub fn expr(&self) -> ExprType {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::ExprType);
		ExprType(ret)
	}
}
