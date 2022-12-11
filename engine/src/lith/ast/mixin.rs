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

use super::{class::ClassInnerKind, item::ItemDef, FieldDeclaration};

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
	Item(ItemDef),
}

impl MixinClassInnerKind {
	pub(crate) fn map_to_class_inner_kind(self) -> ClassInnerKind {
		match self {
			MixinClassInnerKind::Field(field) => ClassInnerKind::Field(field),
			MixinClassInnerKind::Item(item) => ClassInnerKind::Item(item),
		}
	}
}
