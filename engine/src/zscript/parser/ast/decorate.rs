//! DECORATE has a different grammmar near the top-level, and needs elements
//! for representing user-var declarations.

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

use super::{
	super::{ir::*, Span},
	states::StatesDefinition,
};

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum DecorateTopLevelDefinitionKind {
	Actor(ActorDefinition),
	Include(StringConst),
	Const(ConstDefinition),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DecorateTopLevelDefinition {
	pub span: Span,
	#[serde(flatten)]
	pub kind: DecorateTopLevelDefinitionKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DecorateTopLevel {
	pub definitions: Vec<DecorateTopLevelDefinition>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ActorDefinition {
	pub span: Span,
	pub name: Identifier,
	pub ancestor: Option<DottableId>,
	pub replaces: Option<DottableId>,
	pub inners: Vec<ActorInner>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ActorInner {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ActorInnerKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum ActorInnerKind {
	UserVar(UserVarDeclaration),
	DefaultStatement(DefaultStatement),
	States(StatesDefinition),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct UserVarDeclaration {
	pub span: Span,
	/// If this is declaring an array, `::1` holds its size.
	pub var: (Identifier, Option<Expression>),
}
