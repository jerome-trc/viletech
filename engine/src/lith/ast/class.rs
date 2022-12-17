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

use crate::utils::lang::{Identifier, Span};

use super::{item::Item, Resolver, VariableDeclaration};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassDef {
	pub span: Span,
	pub name: Identifier,
	pub ancestors: Vec<Resolver>,
	pub quals: Vec<ClassQualifier>,
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
	Mixin(Identifier),
	Field(VariableDeclaration),
	Item(Item),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassQualifier {
	pub span: Span,
	pub kind: ClassQualKind,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ClassQualKind {
	Abstract,
	Final,
}
