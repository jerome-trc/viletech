//! Lith "items" are like Rust items; function/class definitions, etc.

use serde::Serialize;
use vec1::Vec1;

use crate::utils::lang::{Identifier, Span};

use super::{expr::Expression, Annotation, IntLiteral, Resolver, StatementBlock, TypeExpr};

/// Lith "items" are like Rust items; function/class definitions, etc.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Item {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ItemKind,
	/// Outer annotations only, applied to the entire item.
	pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ItemKind {
	Function(FunctionDeclaration),
	TypeAlias(TypeAlias),
	Constant(Constant),
	Class(ClassDef),
	ClassExt(ClassExtend),
	MixinClass(MixinClassDef),
	Struct(StructDef),
	StructExt(StructExtend),
	Enum(EnumDef),
	EnumExt(EnumExtend),
	Bitfield(BitfieldDef),
	Union(UnionDef),
	UnionExt(UnionExtend),
	MacroInvoc(MacroInvocation),
}

// Sub-modules for code folding by text editors

/// Enumerations, unions, bitfields, type aliases, symbolic constants.
mod misc {
	use super::*;

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct TypeAlias {
		pub span: Span,
		pub name: Identifier,
		pub quals: Vec<DeclQualifier>,
		pub underlying: TypeExpr,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct Constant {
		pub span: Span,
		pub name: Identifier,
		pub quals: Vec<DeclQualifier>,
		pub type_spec: Option<TypeExpr>,
		pub value: Expression,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct EnumDef {
		pub span: Span,
		pub name: Identifier,
		pub quals: Vec<DeclQualifier>,
		pub type_spec: Option<TypeExpr>,
		pub variants: Vec<EnumVariant>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct EnumVariant {
		pub span: Span,
		pub name: Identifier,
		pub init: Option<Expression>,
		/// Outer annotations only.
		pub annotations: Vec<Annotation>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct EnumExtend {
		pub span: Span,
		pub name: Identifier,
		pub inners: Vec<Item>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct UnionDef {
		pub span: Span,
		pub name: Identifier,
		pub quals: Vec<DeclQualifier>,
		pub variants: Vec<UnionVariant>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct UnionVariant {
		pub span: Span,
		pub name: Identifier,
		pub fields: Vec<FieldDeclaration>,
		/// Outer annotations only.
		pub annotations: Vec<Annotation>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct UnionExtend {
		pub span: Span,
		pub name: Identifier,
		pub inners: Vec<Item>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct BitfieldDef {
		pub span: Span,
		pub name: Identifier,
		pub type_spec: TypeExpr,
		pub subfields: Vec<BitfieldBit>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct BitfieldBit {
		pub span: Span,
		pub name: Identifier,
		pub shifts: Vec1<BitfieldBitShift>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub enum BitfieldBitShift {
		Integer(IntLiteral),
		Subfield(Identifier),
	}
}

pub use misc::*;

mod class {
	use super::*;

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct ClassDef {
		pub span: Span,
		pub name: Identifier,
		pub ancestors: Vec<Resolver>,
		pub quals: Vec<DeclQualifier>,
		pub inners: Vec<ClassInner>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct ClassExtend {
		pub span: Span,
		pub name: Identifier,
		pub inners: Vec<ClassInner>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct ClassInner {
		pub span: Span,
		#[serde(flatten)]
		pub kind: ClassInnerKind,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	#[serde(tag = "kind", content = "data")]
	pub enum ClassInnerKind {
		Annotation(Annotation),
		Mixin(Resolver),
		Field(FieldDeclaration),
		Item(Item),
	}
}

pub use class::*;

mod structure {
	use super::*;

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct StructDef {
		pub span: Span,
		pub name: Identifier,
		pub quals: Vec<DeclQualifier>,
		pub inners: Vec<StructInner>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct StructInner {
		pub span: Span,
		#[serde(flatten)]
		pub kind: StructInnerKind,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub enum StructInnerKind {
		Field(FieldDeclaration),
		Item(Item),
		Annotation(Annotation),
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct StructExtend {
		pub span: Span,
		pub name: Identifier,
		pub inners: Vec<StructInner>,
	}
}

pub use structure::*;

mod mixin {
	use super::*;

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct MixinClassDef {
		pub span: Span,
		pub name: Identifier,
		pub inners: Vec<MixinClassInner>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct MixinClassInner {
		pub span: Span,
		#[serde(flatten)]
		pub kind: MixinClassInnerKind,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub enum MixinClassInnerKind {
		Field(FieldDeclaration),
		Item(Item),
	}

	impl MixinClassInnerKind {
		pub(crate) fn map_to_class_inner_kind(self) -> ClassInnerKind {
			match self {
				MixinClassInnerKind::Field(field) => ClassInnerKind::Field(field),
				MixinClassInnerKind::Item(item) => ClassInnerKind::Item(item),
			}
		}
	}
}

pub use mixin::*;

mod inner {
	use super::*;

	#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
	pub struct DeclQualifier {
		pub span: Span,
		#[serde(flatten)]
		pub kind: DeclQualifierKind,
	}

	#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
	pub enum DeclQualifierKind {
		Abstract,
		CEval,
		Final,
		Override,
		Private,
		Protected,
		Public,
		Static,
		Virtual,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct FieldDeclaration {
		pub span: Span,
		pub name: Identifier,
		pub type_spec: TypeExpr,
		pub quals: Vec<DeclQualifier>,
		/// Outer annotations only.
		pub annotations: Vec<Annotation>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct FunctionDeclaration {
		pub span: Span,
		pub name: Identifier,
		pub quals: Vec<DeclQualifier>,
		pub return_type: TypeExpr,
		pub params: Vec<FuncParameter>,
		pub body: Option<StatementBlock>,
	}

	#[derive(Debug, Clone, PartialEq, Serialize)]
	pub struct FuncParameter {
		pub span: Span,
		pub name: Identifier,
		pub quals: Vec<FuncParamQualifier>,
		pub type_spec: TypeExpr,
		pub default: Option<Expression>,
		/// Outer annotations only.
		pub annotations: Vec<Annotation>,
	}

	#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
	pub struct FuncParamQualifier {
		pub span: Span,
		#[serde(flatten)]
		pub kind: FuncParamQualKind,
	}

	#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
	#[serde(tag = "kind", content = "def")]
	pub enum FuncParamQualKind {
		In,
		Out,
		Const,
	}
}

pub use inner::*;

// Item innards ////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacroInvocation {
	pub span: Span,
	pub resolver: Resolver,
	pub inner: String,
}
