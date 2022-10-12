/*

Copyright (C) 2021-2022 Jessica "Gutawer" Russell

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

use crate::zscript::parser::{interner::StringSymbol, ir::*, Span};

use super::{
	class::{ClassInnerKind, StructDefinition},
	states::StatesDefinition,
	Declaration, StaticConstArray,
};

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum MixinClassInnerKind {
	Declaration(Declaration),
	Enum(EnumDefinition),
	Struct(StructDefinition),
	States(StatesDefinition),
	Default(DefaultDefinition),
	Const(ConstDefinition),
	Property(PropertyDefinition),
	Flag(FlagDefinition),
	StaticConstArray(StaticConstArray),
}

impl MixinClassInnerKind {
	pub(crate) fn map_to_class_inner_kind(self) -> ClassInnerKind {
		match self {
			MixinClassInnerKind::Declaration(d) => ClassInnerKind::Declaration(d),
			MixinClassInnerKind::Enum(e) => ClassInnerKind::Enum(e),
			MixinClassInnerKind::Struct(s) => ClassInnerKind::Struct(s),
			MixinClassInnerKind::States(s) => ClassInnerKind::States(s),
			MixinClassInnerKind::Default(d) => ClassInnerKind::Default(d),
			MixinClassInnerKind::Const(c) => ClassInnerKind::Const(c),
			MixinClassInnerKind::Property(p) => ClassInnerKind::Property(p),
			MixinClassInnerKind::Flag(f) => ClassInnerKind::Flag(f),
			MixinClassInnerKind::StaticConstArray(sca) => ClassInnerKind::StaticConstArray(sca),
		}
	}
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MixinClassInner {
	pub span: Span,
	#[serde(flatten)]
	pub kind: MixinClassInnerKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MixinClassDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<MixinClassInner>,
}
