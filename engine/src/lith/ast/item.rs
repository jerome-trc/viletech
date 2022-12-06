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

use super::{Resolver, expr::Expression, decl::{FieldDeclaration, FunctionDeclaration}, class::ClassDef};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemDef<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: ItemDefKind<'inp>,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnionDef<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub variants: Vec<UnionVariant<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnionVariant<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub inners: Vec<FieldDeclaration<'inp>>,
}
