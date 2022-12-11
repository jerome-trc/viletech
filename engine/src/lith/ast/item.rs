//! Lith "items" are like Rust items; function/class definitions, etc.

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use serde::Serialize;

use crate::utils::lang::{Span, Identifier};

use super::{class::ClassDef, expr::Expression, Annotation, CompoundStatement, Resolver, TypeExpr};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemDef {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ItemDefKind,
	pub quals: Vec<DeclQualifier>,
	/// Outer annotations only, applied to the entire item.
	pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ItemDefKind {
	TypeAlias(TypeAlias),
	Constant(Constant),
	Enum(EnumDef),
	Union(UnionDef),
	Function(FunctionDeclaration),
	Class(ClassDef),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypeAlias {
	pub span: Span,
	pub name: Identifier,
	pub resolver: Resolver,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Constant {
	pub span: Span,
	pub name: Identifier,
	pub type_spec: Option<Resolver>,
	pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnumDef {
	pub span: Span,
	pub name: Identifier,
	pub type_spec: Option<Resolver>,
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
pub struct UnionDef {
	pub span: Span,
	pub name: Identifier,
	pub variants: Vec<UnionVariant>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnionVariant {
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<FieldDeclaration>,
	/// Outer annotations only.
	pub annotations: Vec<Annotation>,
}

// Item innards ////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclQualifier {
	pub span: Span,
	#[serde(flatten)]
	pub kind: DeclQualifierKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DeclQualifierKind {
	Abstract,
	Action,
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
	pub type_spec: Resolver,
	pub quals: Vec<DeclQualifier>,
	/// Outer annotations only.
	pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionDeclaration {
	pub span: Span,
	pub name: Identifier,
	pub quals: Vec<DeclQualifier>,
	pub return_type: Option<TypeExpr>,
	pub params: Vec<FuncParameter>,
	pub body: Option<CompoundStatement>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FuncParameter {
	pub span: Span,
	pub quals: Vec<FuncParamQualifier>,
	pub type_spec: TypeExpr,
	pub name: Identifier,
	pub default: Option<Expression>,
	/// Outer annotations only.
	pub annotation: Vec<Annotation>,
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
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PropertyDef {
	pub span: Span,
	pub name: Identifier,
	pub quals: Vec<DeclQualifier>,
	pub type_spec: Option<TypeExpr>,
	pub getter: Option<CompoundStatement>,
	pub setter: Option<CompoundStatement>,
}
