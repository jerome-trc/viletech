//! Abstract syntax tree nodes for representing function declarations and symbolic constants.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syn, SyntaxNode, SyntaxToken};

use super::*;

/// Wraps a node tagged [`Syn::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Item {
	Function(FunctionDecl),
	SymConst(SymConst),
}

impl AstNode for Item {
	type Language = Syn;

	fn can_cast(kind: Syn) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syn::FunctionDecl | Syn::SymConst)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syn::FunctionDecl => Some(Self::Function(FunctionDecl(node))),
			Syn::SymConst => Some(Self::SymConst(SymConst(node))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Self::Function(inner) => inner.syntax(),
			Self::SymConst(inner) => inner.syntax(),
		}
	}
}

// FunctionDecl ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionDecl(SyntaxNode);

simple_astnode!(Syn, FunctionDecl, Syn::FunctionDecl);

impl FunctionDecl {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}

	pub fn params(&self) -> AstResult<ParamList> {
		self.0
			.children()
			.find_map(ParamList::cast)
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn return_type(&self) -> Option<TypeSpec> {
		self.0.children().find_map(TypeSpec::cast)
	}

	pub fn annotations(&self) -> impl Iterator<Item = Annotation> {
		self.0.children().filter_map(Annotation::cast)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		doc_comments(&self.0)
	}
}

// ParamList, Parameter ////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ParamList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamList(pub(super) SyntaxNode);

simple_astnode!(Syn, ParamList, Syn::ParamList);

impl ParamList {
	pub fn iter(&self) -> impl Iterator<Item = Parameter> {
		self.0.children().filter_map(Parameter::cast)
	}
}

/// Wraps a node tagged [`Syn::Parameter`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameter(SyntaxNode);

simple_astnode!(Syn, Parameter, Syn::Parameter);

impl Parameter {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}

	/// The returned token is always tagged [`Syn::KwConst`].
	#[must_use]
	pub fn const_kw(&self) -> Option<SyntaxToken> {
		self.0
			.first_token()
			.filter(|token| token.kind() == Syn::KwConst)
	}

	#[must_use]
	pub fn is_const(&self) -> bool {
		self.const_kw().is_some()
	}

	pub fn type_spec(&self) -> AstResult<TypeSpec> {
		self.0
			.children()
			.find_map(TypeSpec::cast)
			.ok_or(AstError::Missing)
	}

	#[must_use]
	pub fn default(&self) -> Option<Expr> {
		match self.0.last_child() {
			Some(node) => Expr::cast(node),
			None => None,
		}
	}
}

// SymConst ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::SymConst`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SymConst(SyntaxNode);

simple_astnode!(Syn, SymConst, Syn::SymConst);

impl SymConst {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		let mut ident = None;
		let mut eq = None;

		for elem in self.0.children_with_tokens() {
			let Some(token) = elem.into_token() else {
				continue;
			};

			match token.kind() {
				Syn::Ident => ident = Some(token),
				Syn::Eq => eq = Some(token),
				_ => continue,
			}
		}

		let Some(ident) = ident else {
			return Err(AstError::Missing);
		};

		let Some(eq) = eq else {
			return Err(AstError::Incorrect);
		};

		if ident.index() >= eq.index() {
			return Err(AstError::Missing);
		}

		Ok(ident)
	}

	pub fn type_spec(&self) -> AstResult<TypeSpec> {
		TypeSpec::cast(self.0.first_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}

	pub fn expr(&self) -> AstResult<Expr> {
		Expr::cast(self.0.last_child().ok_or(AstError::Missing)?).ok_or(AstError::Incorrect)
	}
}
