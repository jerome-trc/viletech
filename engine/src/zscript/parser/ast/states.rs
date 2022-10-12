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

use vec1::Vec1;

use serde::Serialize;

use super::{
	super::{ir::*, Span},
	CompoundStatement,
};

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum StateLineActionKind {
	Call {
		func: Identifier,
		args: Option<Vec<FunctionCallArg>>,
	},
	Anonymous(CompoundStatement),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StateLineAction {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StateLineActionKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum StateLineMetadataItemKind {
	Bright,
	Fast,
	Slow,
	NoDelay,
	CanRaise,
	Offset(Expression, Expression),
	Light(Vec1<StringConst>),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StateLineMetadataItem {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StateLineMetadataItemKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StateLine {
	pub span: Span,
	pub sprite: NonWhitespace,
	pub frames: NonWhitespace,
	pub duration: Expression,
	pub metadata: Vec<StateLineMetadataItem>,
	pub action: Option<StateLineAction>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum StateGotoTargetKind {
	Unscoped(DottableId),
	Scoped(Identifier, DottableId),
	Super(DottableId),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StateGotoTarget {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StateGotoTargetKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum StatesBodyItemKind {
	Label(NonWhitespace),
	Line(StateLine),
	Stop,
	Wait,
	Fail,
	Loop,
	Goto {
		target: StateGotoTarget,
		offset: Option<Expression>,
	},
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StatesBodyItem {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StatesBodyItemKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StatesDefinition {
	pub span: Span,
	pub opts: Option<Vec1<Identifier>>,
	pub body: Vec<StatesBodyItem>,
}
