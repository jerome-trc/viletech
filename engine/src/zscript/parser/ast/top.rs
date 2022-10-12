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

use crate::zscript::parser::{ir::*, Span};

use super::{
	class::{ClassDefinition, ExtendClass, ExtendStruct, StructDefinition},
	mixin::MixinClassDefinition,
};

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDefinitionKind {
	Class(ClassDefinition),
	Struct(StructDefinition),
	ExtendClass(ExtendClass),
	ExtendStruct(ExtendStruct),
	MixinClass(MixinClassDefinition),
	Enum(EnumDefinition),
	Const(ConstDefinition),
	Include(StringConst),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TopLevelDefinition {
	pub span: Span,
	#[serde(flatten)]
	pub kind: TopLevelDefinitionKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TopLevel {
	pub version: Option<VersionInfo>,
	pub definitions: Vec<TopLevelDefinition>,
}
