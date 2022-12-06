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

use crate::utils::lang::{FileSpan, Identifier};

use super::{class::ClassDef, expr::Expression, CompoundStatement, Resolver, TypeExpr};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemDef<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: ItemDefKind<'inp>,
	pub quals: Vec<DeclQualifier<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ItemDefKind<'inp> {
	TypeAlias(TypeAlias<'inp>),
	Constant(Constant<'inp>),
	Enum(EnumDef<'inp>),
	Union(UnionDef<'inp>),
	Function(FunctionDeclaration<'inp>),
	Class(ClassDef<'inp>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypeAlias<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub resolver: Resolver<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Constant<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub type_spec: Option<Resolver<'inp>>,
	pub value: Expression<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnumDef<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub type_spec: Option<Resolver<'inp>>,
	pub variants: Vec<EnumVariant<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnumVariant<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub init: Option<Expression<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnionDef<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub variants: Vec<UnionVariant<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnionVariant<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub inners: Vec<FieldDeclaration<'inp>>,
}

// Item innards ////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclQualifier<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: DeclQualifierKind,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum DeclQualifierKind {
	Static,
	Private,
	Protected,
	Final,
	Virtual,
	Override,
	Abstract,
	Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldDeclaration<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub type_spec: Resolver<'inp>,
	pub quals: Vec<DeclQualifier<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionDeclaration<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub quals: Vec<DeclQualifier<'inp>>,
	pub return_type: Option<TypeExpr<'inp>>,
	pub params: Vec<FuncParameter<'inp>>,
	pub body: Option<CompoundStatement<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FuncParameter<'inp>(
	pub FileSpan<'inp>,
	pub Vec<FuncParamQualifier<'inp>>,
	pub TypeExpr<'inp>,
	pub Identifier<'inp>,
	pub Option<Expression<'inp>>,
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FuncParamQualifier<'inp> {
	pub span: FileSpan<'inp>,
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
pub struct PropertyDef<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub quals: Vec<DeclQualifier<'inp>>,
	pub type_spec: Option<TypeExpr<'inp>>,
	pub getter: Option<CompoundStatement<'inp>>,
	pub setter: Option<CompoundStatement<'inp>>,
}
