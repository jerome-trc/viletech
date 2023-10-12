//! Abstract syntax tree nodes for representing function declarations and symbolic constants.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syn, SyntaxNode, SyntaxToken};

use super::*;

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
