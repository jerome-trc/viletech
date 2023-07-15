//! AST nodes for representing classes, mixin classes, and structs.

use rowan::ast::AstNode;

use crate::{simple_astnode, AstError, AstResult};

use super::{
	ActionQual, CompoundStat, ConstDef, DefaultBlock, DeprecationQual, DocComment, EnumDef, Expr,
	FlagDef, PropertyDef, StatesBlock, StaticConstStat, Syn, SyntaxNode, SyntaxToken, TypeRef,
	VarName, VersionQual,
};

// ClassDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::ClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassDef(pub(super) SyntaxNode);

simple_astnode!(Syn, ClassDef, Syn::ClassDef);

impl ClassDef {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
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

	pub fn qualifiers(&self) -> impl Iterator<Item = ClassQual> {
		let quals = self
			.0
			.children()
			.find(|node| node.kind() == Syn::ClassQuals)
			.unwrap();

		quals
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syn::KwAbstract => Some(ClassQual::Abstract(elem.into_token().unwrap())),
				Syn::KwNative => Some(ClassQual::Native(elem.into_token().unwrap())),
				Syn::KwPlay => Some(ClassQual::Play(elem.into_token().unwrap())),
				Syn::ReplacesClause => Some(ClassQual::Replaces(ReplacesClause(
					elem.into_node().unwrap(),
				))),
				Syn::KwUi => Some(ClassQual::Ui(elem.into_token().unwrap())),
				Syn::VersionQual => {
					Some(ClassQual::Version(VersionQual(elem.into_node().unwrap())))
				}
				_ => None,
			})
	}

	pub fn innards(&self) -> impl Iterator<Item = ClassInnard> {
		ClassInnard::iter_from_node(self.0.clone())
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
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
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}

	pub fn innards(&self) -> impl Iterator<Item = ClassInnard> {
		ClassInnard::iter_from_node(self.0.clone())
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
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}

	pub fn innards(&self) -> impl Iterator<Item = ClassInnard> {
		ClassInnard::iter_from_node(self.0.clone())
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
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
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}

	pub fn qualifiers(&self) -> impl Iterator<Item = StructQual> {
		let quals = self.0.first_child().unwrap();
		debug_assert_eq!(quals.kind(), Syn::ClassQuals);

		quals
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syn::KwClearScope => Some(StructQual::ClearScope(elem.into_token().unwrap())),
				Syn::KwNative => Some(StructQual::Native(elem.into_token().unwrap())),
				Syn::KwPlay => Some(StructQual::Play(elem.into_token().unwrap())),
				Syn::KwUi => Some(StructQual::Ui(elem.into_token().unwrap())),
				Syn::VersionQual => {
					Some(StructQual::Version(VersionQual(elem.into_node().unwrap())))
				}
				_ => None,
			})
	}

	pub fn innards(&self) -> impl Iterator<Item = StructInnard> {
		StructInnard::iter_from_node(self.0.clone())
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
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
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}

	pub fn innards(&self) -> impl Iterator<Item = StructInnard> {
		StructInnard::iter_from_node(self.0.clone())
	}
}

// ClassQual ///////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ClassQual {
	Replaces(ReplacesClause),
	Abstract(SyntaxToken),
	Play(SyntaxToken),
	Ui(SyntaxToken),
	Native(SyntaxToken),
	Version(VersionQual),
}

/// Wraps a node tagged [`Syn::ReplacesClause`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReplacesClause(SyntaxNode);

simple_astnode!(Syn, ReplacesClause, Syn::ReplacesClause);

impl ReplacesClause {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn replaced(&self) -> AstResult<SyntaxToken> {
		self.0
			.last_token()
			.filter(|token| token.kind() == Syn::Ident)
			.ok_or(AstError::Missing)
	}
}

// ClassInnard /////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ClassInnard {
	Const(ConstDef),
	Enum(EnumDef),
	StaticConst(StaticConstStat),
	Function(FunctionDecl),
	Field(FieldDecl),
	Mixin(MixinStat),
	Default(DefaultBlock),
	States(StatesBlock),
	Property(PropertyDef),
	Flag(FlagDef),
}

impl ClassInnard {
	fn iter_from_node(node: SyntaxNode) -> impl Iterator<Item = ClassInnard> {
		debug_assert!(matches!(
			node.kind(),
			Syn::ClassDef | Syn::ClassExtend | Syn::MixinClassDef
		));

		node.children().filter_map(|node| match node.kind() {
			Syn::ConstDef => Some(ClassInnard::Const(ConstDef(node))),
			Syn::EnumDef => Some(ClassInnard::Enum(EnumDef(node))),
			Syn::StaticConstStat => Some(ClassInnard::StaticConst(StaticConstStat(node))),
			Syn::FunctionDecl => Some(ClassInnard::Function(FunctionDecl(node))),
			Syn::FieldDecl => Some(ClassInnard::Field(FieldDecl(node))),
			Syn::MixinStat => Some(ClassInnard::Mixin(MixinStat(node))),
			Syn::DefaultBlock => Some(ClassInnard::Default(DefaultBlock(node))),
			Syn::StatesBlock => Some(ClassInnard::States(StatesBlock(node))),
			Syn::PropertyDef => Some(ClassInnard::Property(PropertyDef(node))),
			Syn::FlagDef => Some(ClassInnard::Flag(FlagDef(node))),
			_ => None,
		})
	}
}

/// Wraps a node tagged [`Syn::MixinStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MixinStat(SyntaxNode);

simple_astnode!(Syn, MixinStat, Syn::MixinStat);

impl MixinStat {
	/// The returned token is always tagged [`Syn::Ident`].
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
	}
}

// StructQual //////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StructQual {
	Play(SyntaxToken),
	Ui(SyntaxToken),
	Native(SyntaxToken),
	ClearScope(SyntaxToken),
	Version(VersionQual),
}

// StructInnard ////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum StructInnard {
	Const(ConstDef),
	Enum(EnumDef),
	StaticConst(StaticConstStat),
	Function(FunctionDecl),
	Field(FieldDecl),
}

impl StructInnard {
	fn iter_from_node(node: SyntaxNode) -> impl Iterator<Item = StructInnard> {
		debug_assert!(matches!(node.kind(), Syn::StructDef | Syn::StructExtend));

		node.children().filter_map(|node| match node.kind() {
			Syn::ConstDef => Some(StructInnard::Const(ConstDef(node))),
			Syn::EnumDef => Some(StructInnard::Enum(EnumDef(node))),
			Syn::StaticConstStat => Some(StructInnard::StaticConst(StaticConstStat(node))),
			Syn::FunctionDecl => Some(StructInnard::Function(FunctionDecl(node))),
			Syn::FieldDecl => Some(StructInnard::Field(FieldDecl(node))),
			_ => None,
		})
	}
}

// FieldDecl ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FieldDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldDecl(SyntaxNode);

simple_astnode!(Syn, FieldDecl, Syn::FieldDecl);

impl FieldDecl {
	pub fn type_spec(&self) -> AstResult<TypeRef> {
		self.0
			.children()
			.find_map(TypeRef::cast)
			.ok_or(AstError::Missing)
	}

	pub fn names(&self) -> impl Iterator<Item = VarName> {
		self.0.children().filter_map(VarName::cast)
	}

	#[must_use]
	pub fn qualifiers(&self) -> MemberQuals {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::MemberQuals);
		MemberQuals(ret)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}
}

// FunctionDecl ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionDecl(pub(super) SyntaxNode);

simple_astnode!(Syn, FunctionDecl, Syn::FunctionDecl);

impl FunctionDecl {
	#[must_use]
	pub fn qualifiers(&self) -> MemberQuals {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syn::MemberQuals);
		MemberQuals(ret)
	}

	#[must_use]
	pub fn return_types(&self) -> ReturnTypes {
		self.0.children().find_map(ReturnTypes::cast).unwrap()
	}

	#[must_use]
	pub fn param_list(&self) -> AstResult<ParamList> {
		self.0
			.children()
			.find_map(ParamList::cast)
			.ok_or(AstError::Missing)
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
	pub fn body(&self) -> Option<CompoundStat> {
		CompoundStat::cast(self.0.last_child().unwrap())
	}

	#[must_use]
	pub fn is_const(&self) -> bool {
		self.0
			.children_with_tokens()
			.skip_while(|elem| elem.kind() != Syn::ParamList)
			.any(|elem| elem.kind() == Syn::KwConst)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}
}

/// Wraps a node tagged [`Syn::ReturnTypes`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReturnTypes(SyntaxNode);

simple_astnode!(Syn, ReturnTypes, Syn::ReturnTypes);

impl ReturnTypes {
	pub fn iter(&self) -> impl Iterator<Item = TypeRef> {
		self.0.children().filter_map(TypeRef::cast)
	}
}

/// Wraps a node tagged [`Syn::ParamList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParamList(SyntaxNode);

simple_astnode!(Syn, ParamList, Syn::ParamList);

impl ParamList {
	/// Note that the returned iterator yields nothing if this function's
	/// parameter list is just `(void)`.
	pub fn iter(&self) -> impl Iterator<Item = Parameter> {
		self.0.children().filter_map(Parameter::cast)
	}

	/// Returns `true` if this parameter list is only parentheses.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.0.children().next().is_none()
	}

	/// Returns `true` if this parameter list is only the token [`Syn::KwVoid`]
	/// enclosed by parentheses.
	#[must_use]
	pub fn is_void(&self) -> bool {
		self.is_empty()
			&& self
				.0
				.children_with_tokens()
				.find(|elem| elem.kind() == Syn::KwVoid)
				.is_some()
	}
}

// MemberQuals /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syn::MemberQuals`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MemberQuals(SyntaxNode);

simple_astnode!(Syn, MemberQuals, Syn::MemberQuals);

impl MemberQuals {
	pub fn iter(&self) -> impl Iterator<Item = MemberQual> {
		self.0
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syn::DeprecationQual => Some(MemberQual::Deprecation(DeprecationQual(
					elem.into_node().unwrap(),
				))),
				Syn::VersionQual => {
					Some(MemberQual::Version(VersionQual(elem.into_node().unwrap())))
				}
				Syn::ActionQual => Some(MemberQual::Action(ActionQual(elem.into_node().unwrap()))),
				Syn::KwAbstract => Some(MemberQual::Abstract(elem.into_token().unwrap())),
				Syn::KwClearScope => Some(MemberQual::ClearScope(elem.into_token().unwrap())),
				Syn::KwFinal => Some(MemberQual::Final(elem.into_token().unwrap())),
				Syn::KwInternal => Some(MemberQual::Internal(elem.into_token().unwrap())),
				Syn::KwMeta => Some(MemberQual::Meta(elem.into_token().unwrap())),
				Syn::KwNative => Some(MemberQual::Native(elem.into_token().unwrap())),
				Syn::KwOverride => Some(MemberQual::Override(elem.into_token().unwrap())),
				Syn::KwPlay => Some(MemberQual::Play(elem.into_token().unwrap())),
				Syn::KwPrivate => Some(MemberQual::Private(elem.into_token().unwrap())),
				Syn::KwProtected => Some(MemberQual::Protected(elem.into_token().unwrap())),
				Syn::KwReadOnly => Some(MemberQual::ReadOnly(elem.into_token().unwrap())),
				Syn::KwStatic => Some(MemberQual::Static(elem.into_token().unwrap())),
				Syn::KwTransient => Some(MemberQual::Transient(elem.into_token().unwrap())),
				Syn::KwUi => Some(MemberQual::Ui(elem.into_token().unwrap())),
				Syn::KwVarArg => Some(MemberQual::VarArg(elem.into_token().unwrap())),
				Syn::KwVirtual => Some(MemberQual::Virtual(elem.into_token().unwrap())),
				Syn::KwVirtualScope => Some(MemberQual::VirtualScope(elem.into_token().unwrap())),
				_ => None,
			})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum MemberQual {
	Action(ActionQual),
	Deprecation(DeprecationQual),
	Version(VersionQual),
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
	pub fn name(&self) -> AstResult<SyntaxToken> {
		self.0
			.children_with_tokens()
			.find_map(|elem| elem.into_token().filter(|token| token.kind() == Syn::Ident))
			.ok_or(AstError::Missing)
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
