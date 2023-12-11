//! AST nodes for representing classes, mixin classes, and structs.

use rowan::{ast::AstNode, TextRange};

use crate::{simple_astnode, AstError, AstResult};

use super::{
	ActionQual, CompoundStat, ConstDef, DefaultBlock, DeprecationQual, DocComment, EnumDef, Expr,
	FlagDef, PropertyDef, StatesBlock, StaticConstStat, Syntax, SyntaxNode, SyntaxToken, TypeRef,
	VarName, VersionQual,
};

// ClassDef ////////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassDef(pub(super) SyntaxNode);

simple_astnode!(Syntax, ClassDef, Syntax::ClassDef);

impl ClassDef {
	#[must_use]
	pub fn head(&self) -> ClassHead {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::ClassHead);
		ClassHead(ret)
	}

	pub fn innards(&self) -> impl Iterator<Item = ClassInnard> {
		ClassInnard::iter_from_node(self.0.clone())
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}
}

/// Wraps a node tagged [`Syntax::ClassHead`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassHead(SyntaxNode);

simple_astnode!(Syntax, ClassHead, Syntax::ClassHead);

impl ClassHead {
	/// The returned token is always tagged [`Syntax::KwClass`].
	#[must_use]
	pub fn keyword(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwClass)
			})
			.unwrap()
	}

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

	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn parent_class(&self) -> Option<SyntaxToken> {
		let spec = self
			.0
			.children()
			.find_map(|node| Some(node).filter(|node| node.kind() == Syntax::InheritSpec));

		let Some(spec) = spec else {
			return None;
		};
		let ret = spec.last_token().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::Ident);
		Some(ret)
	}

	pub fn qualifiers(&self) -> impl Iterator<Item = ClassQual> {
		self.0
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syntax::KwAbstract => Some(ClassQual::Abstract(elem.into_token().unwrap())),
				Syntax::KwNative => Some(ClassQual::Native(elem.into_token().unwrap())),
				Syntax::KwPlay => Some(ClassQual::Play(elem.into_token().unwrap())),
				Syntax::ReplacesClause => Some(ClassQual::Replaces(ReplacesClause(
					elem.into_node().unwrap(),
				))),
				Syntax::KwUi => Some(ClassQual::Ui(elem.into_token().unwrap())),
				Syntax::VersionQual => {
					Some(ClassQual::Version(VersionQual(elem.into_node().unwrap())))
				}
				_ => None,
			})
	}
}

// ClassExtend /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::ClassExtend`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ClassExtend(pub(super) SyntaxNode);

simple_astnode!(Syntax, ClassExtend, Syntax::ClassExtend);

impl ClassExtend {
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

	pub fn innards(&self) -> impl Iterator<Item = ClassInnard> {
		ClassInnard::iter_from_node(self.0.clone())
	}
}

// MixinClassDef ///////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::MixinClassDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MixinClassDef(pub(super) SyntaxNode);

simple_astnode!(Syntax, MixinClassDef, Syntax::MixinClassDef);

impl MixinClassDef {
	/// `0` is always tagged [`Syntax::KwMixin`]; `1` is always tagged [`Syntax::KwClass`].
	#[must_use]
	pub fn keywords(&self) -> (SyntaxToken, SyntaxToken) {
		let ret0 = self
			.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwMixin)
			})
			.unwrap();

		let ret1 = self
			.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwClass)
			})
			.unwrap();

		(ret0, ret1)
	}

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

	pub fn innards(&self) -> impl Iterator<Item = ClassInnard> {
		ClassInnard::iter_from_node(self.0.clone())
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}
}

// StructDef ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::StructDef`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructDef(pub(super) SyntaxNode);

simple_astnode!(Syntax, StructDef, Syntax::StructDef);

impl StructDef {
	/// The returned token is always tagged [`Syntax::KwStruct`].
	#[must_use]
	pub fn keyword(&self) -> SyntaxToken {
		self.0
			.children_with_tokens()
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwStruct)
			})
			.unwrap()
	}

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

	pub fn qualifiers(&self) -> impl Iterator<Item = StructQual> {
		let quals = self.0.first_child().unwrap();
		debug_assert_eq!(quals.kind(), Syntax::StructQuals);

		quals
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syntax::KwClearScope => Some(StructQual::ClearScope(elem.into_token().unwrap())),
				Syntax::KwNative => Some(StructQual::Native(elem.into_token().unwrap())),
				Syntax::KwPlay => Some(StructQual::Play(elem.into_token().unwrap())),
				Syntax::KwUi => Some(StructQual::Ui(elem.into_token().unwrap())),
				Syntax::VersionQual => {
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

/// Wraps a node tagged [`Syntax::StructExtend`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructExtend(pub(super) SyntaxNode);

simple_astnode!(Syntax, StructExtend, Syntax::StructExtend);

impl StructExtend {
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

impl ClassQual {
	#[must_use]
	pub fn text_range(&self) -> TextRange {
		match self {
			Self::Abstract(inner) | Self::Play(inner) | Self::Ui(inner) | Self::Native(inner) => {
				inner.text_range()
			}
			Self::Replaces(inner) => inner.syntax().text_range(),
			Self::Version(inner) => inner.syntax().text_range(),
		}
	}
}

/// Wraps a node tagged [`Syntax::ReplacesClause`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReplacesClause(SyntaxNode);

simple_astnode!(Syntax, ReplacesClause, Syntax::ReplacesClause);

impl ReplacesClause {
	/// The returned token is always tagged [`Syntax::Ident`].
	pub fn replaced(&self) -> AstResult<SyntaxToken> {
		self.0
			.last_token()
			.filter(|token| token.kind() == Syntax::Ident)
			.ok_or(AstError::Missing)
	}
}

// ClassInnard /////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ClassInnard {
	Const(ConstDef),
	Enum(EnumDef),
	Struct(StructDef),
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
			Syntax::ClassDef | Syntax::ClassExtend | Syntax::MixinClassDef
		));

		node.children().filter_map(|node| match node.kind() {
			Syntax::ConstDef => Some(ClassInnard::Const(ConstDef(node))),
			Syntax::EnumDef => Some(ClassInnard::Enum(EnumDef(node))),
			Syntax::StructDef => Some(ClassInnard::Struct(StructDef(node))),
			Syntax::StaticConstStat => Some(ClassInnard::StaticConst(StaticConstStat(node))),
			Syntax::FunctionDecl => Some(ClassInnard::Function(FunctionDecl(node))),
			Syntax::FieldDecl => Some(ClassInnard::Field(FieldDecl(node))),
			Syntax::MixinStat => Some(ClassInnard::Mixin(MixinStat(node))),
			Syntax::DefaultBlock => Some(ClassInnard::Default(DefaultBlock(node))),
			Syntax::StatesBlock => Some(ClassInnard::States(StatesBlock(node))),
			Syntax::PropertyDef => Some(ClassInnard::Property(PropertyDef(node))),
			Syntax::FlagDef => Some(ClassInnard::Flag(FlagDef(node))),
			_ => None,
		})
	}
}

/// Wraps a node tagged [`Syntax::MixinStat`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MixinStat(SyntaxNode);

simple_astnode!(Syntax, MixinStat, Syntax::MixinStat);

impl MixinStat {
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

impl StructQual {
	#[must_use]
	pub fn text_range(&self) -> TextRange {
		match self {
			Self::Play(inner) | Self::Ui(inner) | Self::Native(inner) | Self::ClearScope(inner) => {
				inner.text_range()
			}
			Self::Version(inner) => inner.syntax().text_range(),
		}
	}
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
		debug_assert!(matches!(
			node.kind(),
			Syntax::StructDef | Syntax::StructExtend
		));

		node.children().filter_map(|node| match node.kind() {
			Syntax::ConstDef => Some(StructInnard::Const(ConstDef(node))),
			Syntax::EnumDef => Some(StructInnard::Enum(EnumDef(node))),
			Syntax::StaticConstStat => Some(StructInnard::StaticConst(StaticConstStat(node))),
			Syntax::FunctionDecl => Some(StructInnard::Function(FunctionDecl(node))),
			Syntax::FieldDecl => Some(StructInnard::Field(FieldDecl(node))),
			_ => None,
		})
	}
}

// FieldDecl ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::FieldDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FieldDecl(SyntaxNode);

simple_astnode!(Syntax, FieldDecl, Syntax::FieldDecl);

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
		debug_assert_eq!(ret.kind(), Syntax::MemberQuals);
		MemberQuals(ret)
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}
}

// FunctionDecl ////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::FunctionDecl`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FunctionDecl(pub(super) SyntaxNode);

simple_astnode!(Syntax, FunctionDecl, Syntax::FunctionDecl);

impl FunctionDecl {
	#[must_use]
	pub fn qualifiers(&self) -> MemberQuals {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::MemberQuals);
		MemberQuals(ret)
	}

	#[must_use]
	pub fn return_types(&self) -> ReturnTypes {
		self.0.children().find_map(ReturnTypes::cast).unwrap()
	}

	pub fn param_list(&self) -> AstResult<ParamList> {
		self.0
			.children()
			.find_map(ParamList::cast)
			.ok_or(AstError::Missing)
	}

	/// The returned token is always tagged [`Syntax::Ident`].
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
	pub fn body(&self) -> Option<CompoundStat> {
		CompoundStat::cast(self.0.last_child().unwrap())
	}

	#[must_use]
	pub fn is_const(&self) -> bool {
		self.const_keyword().is_some()
	}

	/// The returned token is always tagged [`Syntax::KwConst`].
	#[must_use]
	pub fn const_keyword(&self) -> Option<SyntaxToken> {
		self.0
			.children_with_tokens()
			.skip_while(|elem| elem.kind() != Syntax::ParamList)
			.take_while(|elem| !matches!(elem.kind(), Syntax::Semicolon | Syntax::CompoundStat))
			.find_map(|elem| {
				elem.into_token()
					.filter(|token| token.kind() == Syntax::KwConst)
			})
	}

	pub fn docs(&self) -> impl Iterator<Item = DocComment> {
		super::doc_comments(&self.0)
	}
}

/// Wraps a node tagged [`Syntax::ReturnTypes`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReturnTypes(SyntaxNode);

simple_astnode!(Syntax, ReturnTypes, Syntax::ReturnTypes);

impl ReturnTypes {
	pub fn iter(&self) -> impl Iterator<Item = TypeRef> {
		self.0.children().filter_map(TypeRef::cast)
	}
}

/// Wraps a node tagged [`Syntax::ParamList`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParamList(SyntaxNode);

simple_astnode!(Syntax, ParamList, Syntax::ParamList);

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

	/// Returns `true` if this parameter list is only the token [`Syntax::KwVoid`]
	/// enclosed by parentheses.
	#[must_use]
	pub fn is_void(&self) -> bool {
		self.is_empty()
			&& self
				.0
				.children_with_tokens()
				.any(|elem| elem.kind() == Syntax::KwVoid)
	}

	#[must_use]
	pub fn varargs(&self) -> bool {
		self.0.children_with_tokens().any(|elem| {
			elem.into_token()
				.is_some_and(|token| token.kind() == Syntax::Dot3)
		})
	}
}

// MemberQuals /////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::MemberQuals`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MemberQuals(SyntaxNode);

simple_astnode!(Syntax, MemberQuals, Syntax::MemberQuals);

impl MemberQuals {
	pub fn iter(&self) -> impl Iterator<Item = MemberQual> {
		self.0
			.children_with_tokens()
			.filter_map(|elem| match elem.kind() {
				Syntax::DeprecationQual => Some(MemberQual::Deprecation(DeprecationQual(
					elem.into_node().unwrap(),
				))),
				Syntax::VersionQual => {
					Some(MemberQual::Version(VersionQual(elem.into_node().unwrap())))
				}
				Syntax::ActionQual => {
					Some(MemberQual::Action(ActionQual(elem.into_node().unwrap())))
				}
				Syntax::KwAbstract => Some(MemberQual::Abstract(elem.into_token().unwrap())),
				Syntax::KwClearScope => Some(MemberQual::ClearScope(elem.into_token().unwrap())),
				Syntax::KwFinal => Some(MemberQual::Final(elem.into_token().unwrap())),
				Syntax::KwInternal => Some(MemberQual::Internal(elem.into_token().unwrap())),
				Syntax::KwMeta => Some(MemberQual::Meta(elem.into_token().unwrap())),
				Syntax::KwNative => Some(MemberQual::Native(elem.into_token().unwrap())),
				Syntax::KwOverride => Some(MemberQual::Override(elem.into_token().unwrap())),
				Syntax::KwPlay => Some(MemberQual::Play(elem.into_token().unwrap())),
				Syntax::KwPrivate => Some(MemberQual::Private(elem.into_token().unwrap())),
				Syntax::KwProtected => Some(MemberQual::Protected(elem.into_token().unwrap())),
				Syntax::KwReadOnly => Some(MemberQual::ReadOnly(elem.into_token().unwrap())),
				Syntax::KwStatic => Some(MemberQual::Static(elem.into_token().unwrap())),
				Syntax::KwTransient => Some(MemberQual::Transient(elem.into_token().unwrap())),
				Syntax::KwUi => Some(MemberQual::Ui(elem.into_token().unwrap())),
				Syntax::KwVarArg => Some(MemberQual::VarArg(elem.into_token().unwrap())),
				Syntax::KwVirtual => Some(MemberQual::Virtual(elem.into_token().unwrap())),
				Syntax::KwVirtualScope => {
					Some(MemberQual::VirtualScope(elem.into_token().unwrap()))
				}
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
	/// Only applicable to [functions](FunctionDecl).
	Abstract(SyntaxToken),
	ClearScope(SyntaxToken),
	/// Only applicable to [functions](FunctionDecl).
	Final(SyntaxToken),
	/// Only applicable to [fields](FieldDecl).
	Internal(SyntaxToken),
	/// Only applicable to [fields](FieldDecl).
	Meta(SyntaxToken),
	Native(SyntaxToken),
	/// Only applicable to [functions](FunctionDecl).
	Override(SyntaxToken),
	Play(SyntaxToken),
	Private(SyntaxToken),
	Protected(SyntaxToken),
	/// Only applicable to [fields](FieldDecl).
	ReadOnly(SyntaxToken),
	/// Only applicable to [functions](FunctionDecl).
	Static(SyntaxToken),
	/// Only applicable to [fields](FieldDecl).
	Transient(SyntaxToken),
	Ui(SyntaxToken),
	/// Only applicable to [functions](FunctionDecl).
	VarArg(SyntaxToken),
	/// Only applicable to [functions](FunctionDecl).
	Virtual(SyntaxToken),
	/// Only applicable to [functions](FunctionDecl).
	VirtualScope(SyntaxToken),
}

impl MemberQual {
	#[must_use]
	pub fn text_range(&self) -> TextRange {
		match self {
			Self::Action(inner) => inner.syntax().text_range(),
			Self::Deprecation(inner) => inner.syntax().text_range(),
			Self::Version(inner) => inner.syntax().text_range(),
			Self::Abstract(inner)
			| Self::ClearScope(inner)
			| Self::Final(inner)
			| Self::Internal(inner)
			| Self::Meta(inner)
			| Self::Native(inner)
			| Self::Override(inner)
			| Self::Play(inner)
			| Self::Private(inner)
			| Self::Protected(inner)
			| Self::ReadOnly(inner)
			| Self::Static(inner)
			| Self::Transient(inner)
			| Self::Ui(inner)
			| Self::VarArg(inner)
			| Self::Virtual(inner)
			| Self::VirtualScope(inner) => inner.text_range(),
		}
	}

	#[must_use]
	pub fn kind(&self) -> Syntax {
		match self {
			Self::Action(inner) => inner.syntax().kind(),
			Self::Deprecation(inner) => inner.syntax().kind(),
			Self::Version(inner) => inner.syntax().kind(),
			Self::Abstract(inner)
			| Self::ClearScope(inner)
			| Self::Final(inner)
			| Self::Internal(inner)
			| Self::Meta(inner)
			| Self::Native(inner)
			| Self::Override(inner)
			| Self::Play(inner)
			| Self::Private(inner)
			| Self::Protected(inner)
			| Self::ReadOnly(inner)
			| Self::Static(inner)
			| Self::Transient(inner)
			| Self::Ui(inner)
			| Self::VarArg(inner)
			| Self::Virtual(inner)
			| Self::VirtualScope(inner) => inner.kind(),
		}
	}
}

#[derive(Debug, Default)]
pub struct MemberQualSet {
	// Note that as of GZDoom 4.10.0, any and all repeats are accepted silently
	// by the compiler, even if it's a repeated `version` or `action` qualifier.
	pub q_action: Option<ActionQual>,
	pub q_deprecation: Option<DeprecationQual>,
	pub q_version: Option<VersionQual>,
	pub q_abstract: Option<SyntaxToken>,
	pub q_clearscope: Option<SyntaxToken>,
	pub q_final: Option<SyntaxToken>,
	pub q_internal: Option<SyntaxToken>,
	pub q_meta: Option<SyntaxToken>,
	pub q_native: Option<SyntaxToken>,
	pub q_override: Option<SyntaxToken>,
	pub q_play: Option<SyntaxToken>,
	pub q_private: Option<SyntaxToken>,
	pub q_protected: Option<SyntaxToken>,
	pub q_readonly: Option<SyntaxToken>,
	pub q_static: Option<SyntaxToken>,
	pub q_transient: Option<SyntaxToken>,
	pub q_ui: Option<SyntaxToken>,
	pub q_vararg: Option<SyntaxToken>,
	pub q_virtual: Option<SyntaxToken>,
	pub q_virtualscope: Option<SyntaxToken>,
}

impl MemberQualSet {
	/// The first argument to `F` is the text range of the previous qualifier.
	pub fn new<F>(quals: &MemberQuals, mut repeat_handler: F) -> Self
	where
		F: FnMut(TextRange, &MemberQual),
	{
		let mut ret = Self::default();

		for qual in quals.iter() {
			match qual.clone() {
				MemberQual::Action(inner) => {
					if let Some(prev) = &ret.q_action {
						repeat_handler(prev.syntax().text_range(), &qual)
					}

					ret.q_action = Some(inner);
				}
				MemberQual::Deprecation(inner) => {
					if let Some(prev) = &ret.q_deprecation {
						repeat_handler(prev.syntax().text_range(), &qual)
					}

					ret.q_deprecation = Some(inner);
				}
				MemberQual::Version(inner) => {
					if let Some(prev) = &ret.q_version {
						repeat_handler(prev.syntax().text_range(), &qual)
					}

					ret.q_version = Some(inner);
				}
				MemberQual::Abstract(inner) => {
					if let Some(prev) = &ret.q_abstract {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_abstract = Some(inner);
				}
				MemberQual::ClearScope(inner) => {
					if let Some(prev) = &ret.q_clearscope {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_clearscope = Some(inner);
				}
				MemberQual::Final(inner) => {
					if let Some(prev) = &ret.q_final {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_final = Some(inner);
				}
				MemberQual::Internal(inner) => {
					if let Some(prev) = &ret.q_internal {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_internal = Some(inner);
				}
				MemberQual::Meta(inner) => {
					if let Some(prev) = &ret.q_meta {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_meta = Some(inner);
				}
				MemberQual::Native(inner) => {
					if let Some(prev) = &ret.q_native {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_native = Some(inner);
				}
				MemberQual::Override(inner) => {
					if let Some(prev) = &ret.q_override {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_override = Some(inner);
				}
				MemberQual::Play(inner) => {
					if let Some(prev) = &ret.q_play {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_play = Some(inner);
				}
				MemberQual::Private(inner) => {
					if let Some(prev) = &ret.q_private {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_private = Some(inner);
				}
				MemberQual::Protected(inner) => {
					if let Some(prev) = &ret.q_protected {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_protected = Some(inner);
				}
				MemberQual::ReadOnly(inner) => {
					if let Some(prev) = &ret.q_readonly {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_readonly = Some(inner);
				}
				MemberQual::Static(inner) => {
					if let Some(prev) = &ret.q_static {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_static = Some(inner);
				}
				MemberQual::Transient(inner) => {
					if let Some(prev) = &ret.q_transient {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_transient = Some(inner);
				}
				MemberQual::Ui(inner) => {
					if let Some(prev) = &ret.q_ui {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_ui = Some(inner);
				}
				MemberQual::VarArg(inner) => {
					if let Some(prev) = &ret.q_vararg {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_vararg = Some(inner);
				}
				MemberQual::Virtual(inner) => {
					if let Some(prev) = &ret.q_virtual {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_virtual = Some(inner);
				}
				MemberQual::VirtualScope(inner) => {
					if let Some(prev) = &ret.q_virtualscope {
						repeat_handler(prev.text_range(), &qual)
					}

					ret.q_virtualscope = Some(inner);
				}
			}
		}

		ret
	}
}

// Parameter ///////////////////////////////////////////////////////////////////

/// Wraps a node tagged [`Syntax::Parameter`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Parameter(SyntaxNode);

simple_astnode!(Syntax, Parameter, Syntax::Parameter);

impl Parameter {
	#[must_use]
	pub fn type_spec(&self) -> TypeRef {
		let ret = self.0.first_child().unwrap();
		debug_assert_eq!(ret.kind(), Syntax::TypeRef);
		TypeRef(ret)
	}

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

	#[must_use]
	pub fn default(&self) -> Option<Expr> {
		let ret = self.0.last_child().unwrap();
		Expr::cast(ret)
	}

	#[must_use]
	pub fn is_in(&self) -> bool {
		self.0
			.children_with_tokens()
			.any(|elem| elem.kind() == Syntax::KwIn)
	}

	#[must_use]
	pub fn is_out(&self) -> bool {
		self.0
			.children_with_tokens()
			.any(|elem| elem.kind() == Syntax::KwOut)
	}
}
