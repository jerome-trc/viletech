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

use super::{class::ClassInnerKind, decl::FieldDeclaration, item::ItemDef};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MixinClassDef<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub inners: Vec<MixinClassInner<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MixinClassInner<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: MixinClassInnerKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MixinClassInnerKind<'inp> {
	Field(FieldDeclaration<'inp>),
	Item(ItemDef<'inp>),
}

impl<'inp> MixinClassInnerKind<'inp> {
	pub(crate) fn map_to_class_inner_kind(self) -> ClassInnerKind<'inp> {
		match self {
			MixinClassInnerKind::Field(field) => ClassInnerKind::Field(field),
			MixinClassInnerKind::Item(item) => ClassInnerKind::Item(item),
		}
	}
}
