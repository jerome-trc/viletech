//! Abstract syntax tree nodes for representing function declarations and symbolic constants.

use doomfront::{rowan::ast::AstNode, simple_astnode, AstError, AstResult};

use crate::{Syntax, SyntaxNode, SyntaxToken};

use super::*;

/// Wraps a node tagged [`Syntax::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Item {
	Function(FunctionDecl),
	SymConst(SymConst),
}

impl AstNode for Item {
	type Language = Syntax;

	fn can_cast(kind: Syntax) -> bool
	where
		Self: Sized,
	{
		matches!(kind, Syntax::FunctionDecl | Syntax::SymConst)
	}

	fn cast(node: SyntaxNode) -> Option<Self>
	where
		Self: Sized,
	{
		match node.kind() {
			Syntax::FunctionDecl => Some(Self::Function(FunctionDecl(node))),
			Syntax::SymConst => Some(Self::SymConst(SymConst(node))),
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

/// Wraps a node tagged [`Syntax::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionDecl(SyntaxNode);

simple_astnode!(Syntax, FunctionDecl, Syntax::FunctionDecl);

impl FunctionDecl {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
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

	/// The returned node is always tagged [`Syntax::FunctionBody`].
	#[must_use]
	pub fn body(&self) -> Option<FunctionBody> {
		self.0.children().find_map(FunctionBody::cast)
	}

	pub fn annotations(&self) -> impl Iterator<Item = Annotation> {
		self.0.children().filter_map(Annotation::cast)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		doc_comments(&self.0)
	}
}

// ParamList, Parameter ////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ParamList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamList(pub(super) SyntaxNode);

simple_astnode!(Syntax, ParamList, Syntax::ParamList);

impl ParamList {
	/// The returned token is always tagged [`Syntax::Dot3`].
	#[must_use]
	pub fn dot3(&self) -> Option<SyntaxToken> {
		if self.iter().next().is_some() {
			return None;
		}

		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|t| t.kind() == Syntax::Dot3))
	}

	#[must_use]
	pub fn intrinsic_params(&self) -> bool {
		self.dot3().is_some()
	}

	pub fn iter(&self) -> impl Iterator<Item = Parameter> {
		self.0.children().filter_map(Parameter::cast)
	}
}

/// Wraps a node tagged [`Syntax::Parameter`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameter(SyntaxNode);

simple_astnode!(Syntax, Parameter, Syntax::Parameter);

impl Parameter {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::Ident)
			})
			.ok_or(AstError::Missing)
	}

	/// The returned token is always tagged [`Syntax::KwConst`].
	#[must_use]
	pub fn const_kw(&self) -> Option<SyntaxToken> {
		self.0
			.first_token()
			.filter(|token| token.kind() == Syntax::KwConst)
	}

	#[must_use]
	pub fn is_const(&self) -> bool {
		self.const_kw().is_some()
	}

	#[must_use]
	pub fn ref_spec(&self) -> ParamRefSpec {
		let mut amp = None;
		let mut kw_var = None;

		for elem in self.0.children_with_tokens() {
			match elem.kind() {
				Syntax::Ampersand => amp = elem.into_token(),
				Syntax::KwVar => kw_var = elem.into_token(),
				Syntax::Ident => break,
				_ => continue,
			}
		}

		match (amp, kw_var) {
			(Some(t0), Some(t1)) => ParamRefSpec::RefVar(t0, t1),
			(Some(t0), None) => ParamRefSpec::Ref(t0),
			_ => ParamRefSpec::None,
		}
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

/// See [`Parameter::ref_spec`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamRefSpec {
	None,
	Ref(SyntaxToken),
	RefVar(SyntaxToken, SyntaxToken),
}

/// Wraps a node tagged [`Syntax::FunctionBody`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionBody(SyntaxNode);

simple_astnode!(Syntax, FunctionBody, Syntax::FunctionBody);

impl FunctionBody {
	pub fn innards(&self) -> impl Iterator<Item = CoreElement> {
		self.0.children().filter_map(CoreElement::cast)
	}
}

// SymConst ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::SymConst`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SymConst(SyntaxNode);

simple_astnode!(Syntax, SymConst, Syntax::SymConst);

impl SymConst {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		let mut ident = None;
		let mut eq = None;

		for elem in self.0.children_with_tokens() {
			let Some(token) = elem.into_token() else {
				continue;
			};

			match token.kind() {
				Syntax::Ident => ident = Some(token),
				Syntax::Eq => eq = Some(token),
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

	pub fn annotations(&self) -> impl Iterator<Item = Annotation> {
		self.0.children().filter_map(Annotation::cast)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		doc_comments(&self.0)
	}
}
