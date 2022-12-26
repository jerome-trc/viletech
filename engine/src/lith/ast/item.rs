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
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use serde::Serialize;

use crate::utils::lang::{Identifier, Span};

use super::{class::ClassDef, expr::Expression, Annotation, Resolver, StatementBlock, TypeExpr};

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
	TypeAlias(TypeAlias),
	Constant(Constant),
	Enum(EnumDef),
	Union(UnionDef),
	Function(FunctionDeclaration),
	Class(ClassDef),
	MacroInvoc(MacroInvocation),
}

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
	pub quals: Vec<DeclQualifier>,
	pub inners: Vec<VariableDeclaration>,
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
pub struct VariableDeclaration {
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
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PropertyDef {
	pub span: Span,
	pub name: Identifier,
	pub quals: Vec<DeclQualifier>,
	pub type_spec: Option<TypeExpr>,
	pub getter: Option<StatementBlock>,
	pub setter: Option<StatementBlock>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacroInvocation {
	pub span: Span,
	pub inner: String,
}
