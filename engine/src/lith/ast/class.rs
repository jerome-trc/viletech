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

use super::{item::ItemDef, FieldDeclaration, Resolver};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassDef<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub ancestors: Vec<Resolver<'inp>>,
	pub quals: Vec<ClassQualifier<'inp>>,
	pub inners: Vec<ClassInner<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassExtend<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub inners: Vec<ClassInner<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassInner<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: ClassInnerKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ClassInnerKind<'inp> {
	Mixin(Identifier<'inp>),
	Field(FieldDeclaration<'inp>),
	Item(ItemDef<'inp>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassQualifier<'inp> {
	pub span: FileSpan<'inp>,
	pub kind: ClassQualKind,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ClassQualKind {
	Abstract,
	Final,
}
