//! AST nodes for representing classes, mixin classes, and structs.

use rowan::ast::AstNode;

use crate::simple_astnode;

use super::{Expr, Syn, SyntaxNode, SyntaxToken, TypeRef};

// ClassDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassDef(pub(super) SyntaxNode);

simple_astnode!(Syn, ClassDef, Syn::ClassDef);

impl ClassDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// The returned token is always tagged [`Syn::Ident`].
	pub fn parent_class(&self) -> Option<SyntaxToken> {
		let spec = self
			.0
			.children()
			.find_map(|node| Some(node).filter(|node| node.kind() == Syn::InheritSpec));

		let Some(spec) = spec else { return None; };
		let ret = spec.last_token().unwrap();
		debug_assert_eq!(ret.kind(), Syn::Ident);
		Some(ret)
	}

	/// All returned tokens are tagged [`Syn::DocComment`].
	pub fn docs(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() == Syn::DocComment)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::DocComment)
			})
	}
}

// ClassExtend /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassExtend`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassExtend(pub(super) SyntaxNode);

simple_astnode!(Syn, ClassExtend, Syn::ClassExtend);

impl ClassExtend {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

// MixinClassDef ///////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::MixinClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MixinClassDef(pub(super) SyntaxNode);

simple_astnode!(Syn, MixinClassDef, Syn::MixinClassDef);

impl MixinClassDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// All returned tokens are tagged [`Syn::DocComment`].
	pub fn docs(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() == Syn::DocComment)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::DocComment)
			})
	}
}

// StructDef ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StructDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructDef(pub(super) SyntaxNode);

simple_astnode!(Syn, StructDef, Syn::StructDef);

impl StructDef {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// All returned tokens are tagged [`Syn::DocComment`].
	pub fn docs(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() == Syn::DocComment)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::DocComment)
			})
	}
}

// StructExtend ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::StructExtend`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructExtend(pub(super) SyntaxNode);

simple_astnode!(Syn, StructExtend, Syn::StructExtend);

impl StructExtend {
	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}
}

// FieldDecl ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FieldDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldDecl(SyntaxNode);

simple_astnode!(Syn, FieldDecl, Syn::FieldDecl);

impl FieldDecl {
	/// All returned tokens are tagged [`Syn::DocComment`].
	pub fn docs(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() == Syn::DocComment)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::DocComment)
			})
	}
}

// FunctionDecl ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionDecl(pub(super) SyntaxNode);

simple_astnode!(Syn, FunctionDecl, Syn::FunctionDecl);

impl FunctionDecl {
	pub fn qualifiers(&self) -> impl Iterator<Item = MemberQual> {
		let quals = self.0.first_child().unwrap();
		debug_assert_eq!(quals.kind(), Syn::MemberQuals);

		quals.children_with_tokens().map(|elem| match elem.kind() {
			Syn::DeprecationQual => MemberQual::Deprecation(elem.into_node().unwrap()),
			Syn::VersionQual => MemberQual::Version(elem.into_node().unwrap()),
			Syn::ActionQual => MemberQual::Action(elem.into_node().unwrap()),
			Syn::KwAbstract => MemberQual::Abstract(elem.into_token().unwrap()),
			Syn::KwClearScope => MemberQual::ClearScope(elem.into_token().unwrap()),
			Syn::KwFinal => MemberQual::Final(elem.into_token().unwrap()),
			Syn::KwInternal => MemberQual::Internal(elem.into_token().unwrap()),
			Syn::KwMeta => MemberQual::Meta(elem.into_token().unwrap()),
			Syn::KwNative => MemberQual::Native(elem.into_token().unwrap()),
			Syn::KwOverride => MemberQual::Override(elem.into_token().unwrap()),
			Syn::KwPlay => MemberQual::Play(elem.into_token().unwrap()),
			Syn::KwPrivate => MemberQual::Private(elem.into_token().unwrap()),
			Syn::KwProtected => MemberQual::Protected(elem.into_token().unwrap()),
			Syn::KwReadOnly => MemberQual::ReadOnly(elem.into_token().unwrap()),
			Syn::KwStatic => MemberQual::Static(elem.into_token().unwrap()),
			Syn::KwTransient => MemberQual::Transient(elem.into_token().unwrap()),
			Syn::KwUi => MemberQual::Ui(elem.into_token().unwrap()),
			Syn::KwVarArg => MemberQual::VarArg(elem.into_token().unwrap()),
			Syn::KwVirtual => MemberQual::Virtual(elem.into_token().unwrap()),
			Syn::KwVirtualScope => MemberQual::VirtualScope(elem.into_token().unwrap()),
			_ => unreachable!(),
		})
	}

	pub fn return_types(&self) -> impl Iterator<Item = TypeRef> {
		let rettypes = self
			.0
			.children()
			.find(|node| node.kind() == Syn::ReturnTypes)
			.unwrap();

		rettypes.children().map(|node| {
			debug_assert_eq!(node.kind(), Syn::TypeRef);
			TypeRef(node)
		})
	}

	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	/// Note that the returned iterator yields nothing if this function's
	/// parameter list is just `(void)`.
	pub fn params(&self) -> impl Iterator<Item = Parameter> {
		let paramlist = self
			.0
			.children()
			.find(|node| node.kind() == Syn::ParamList)
			.unwrap();

		paramlist.children().map(|node| {
			debug_assert_eq!(node.kind(), Syn::Parameter);
			Parameter(node)
		})
	}

	/// All returned tokens are tagged [`Syn::DocComment`].
	pub fn docs(&self) -> impl Iterator<Item = SyntaxToken> {
		self.0
			.children_with_tokens()
			.take_while(|elem| elem.kind() == Syn::DocComment)
			.filter_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syn::DocComment)
			})
	}
}

// MemberQual //////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MemberQual {
	Action(SyntaxNode),
	Deprecation(SyntaxNode),
	Version(SyntaxNode),
	Abstract(SyntaxToken),
	ClearScope(SyntaxToken),
	Final(SyntaxToken),
	Internal(SyntaxToken),
	Meta(SyntaxToken),
	Native(SyntaxToken),
	Override(SyntaxToken),
	Play(SyntaxToken),
	Private(SyntaxToken),
	Protected(SyntaxToken),
	ReadOnly(SyntaxToken),
	Static(SyntaxToken),
	Transient(SyntaxToken),
	Ui(SyntaxToken),
	VarArg(SyntaxToken),
	Virtual(SyntaxToken),
	VirtualScope(SyntaxToken),
}

// Parameter ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::Parameter`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Parameter(SyntaxNode);

simple_astnode!(Syn, Parameter, Syn::Parameter);

impl Parameter {
	#[must_use]
	pub fn type_spec(&self) -> TypeRef {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::TypeRef);
		TypeRef(ret)
	}

	/// The returned token is always tagged [`Syn::Ident`].
	#[must_use]
	pub fn name(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.unwrap()
	}

	#[must_use]
	pub fn default(&self) -> Option<Expr> {
		let ret = self.0.last_child().unwrap();
		Expr::cast(ret)
	}

	#[must_use]
	pub fn is_in(&self) -> bool {
		self.0
			.children_with_tokens()
			.any(|elem| elem.kind() == Syn::KwIn)
	}

	#[must_use]
	pub fn is_out(&self) -> bool {
		self.0
			.children_with_tokens()
			.any(|elem| elem.kind() == Syn::KwOut)
	}
}
