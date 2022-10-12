//! Also includes AST components for defining ZScript structs.

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

use super::{states::StatesDefinition, Declaration, StaticConstArray};

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ClassMetadataItemKind {
	Abstract,
	Native,
	UI,
	Play,
	Version(StringConst),
	Replaces(DottableId),
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ClassMetadataItem {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ClassMetadataItemKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum ClassInnerKind {
	Declaration(Declaration),
	Enum(EnumDefinition),
	Struct(StructDefinition),
	States(StatesDefinition),
	Default(DefaultDefinition),
	Const(ConstDefinition),
	Property(PropertyDefinition),
	Flag(FlagDefinition),
	StaticConstArray(StaticConstArray),
	Mixin(Identifier),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ClassInner {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ClassInnerKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ClassDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub ancestor: Option<DottableId>,
	pub metadata: Vec<ClassMetadataItem>,
	pub inners: Vec<ClassInner>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ExtendClass {
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<ClassInner>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StructMetadataItemKind {
	ClearScope,
	Native,
	UI,
	Play,
	Version(StringConst),
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct StructMetadataItem {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StructMetadataItemKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum StructInnerKind {
	Declaration(Declaration),
	Enum(EnumDefinition),
	Const(ConstDefinition),
	StaticConstArray(StaticConstArray),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StructInner {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StructInnerKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StructDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub metadata: Vec<StructMetadataItem>,
	pub inners: Vec<StructInner>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ExtendStruct {
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<StructInner>,
}
