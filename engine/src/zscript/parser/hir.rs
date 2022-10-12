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

use std::collections::HashMap;

use bitflags::bitflags;
use vec1::Vec1;

use super::interner::{NameSymbol, StringSymbol};
use super::ir::*;
use super::Span;

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDefinitionKind {
	Class(ClassDefinition),
	Struct(StructDefinition),
	MixinClass(MixinClassDefinition),
	Enum(EnumDefinition),
	Const(ConstDefinition),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TopLevelDefinition {
	pub span: Span,
	pub archive_num: usize,
	#[serde(flatten)]
	pub kind: TopLevelDefinitionKind,
}

impl TopLevelDefinition {
	pub fn name(&self) -> &Identifier {
		match &self.kind {
			TopLevelDefinitionKind::Class(c) => &c.name,
			TopLevelDefinitionKind::Struct(s) => &s.name,
			TopLevelDefinitionKind::MixinClass(m) => &m.name,
			TopLevelDefinitionKind::Enum(e) => &e.name,
			TopLevelDefinitionKind::Const(c) => &c.name,
		}
	}
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TopLevel {
	pub definitions: HashMap<NameSymbol, Vec1<TopLevelDefinition>>,
}

bitflags! {
	#[derive(Serialize)]
	pub struct ClassDefinitionFlags: u8 {
		const ABSTRACT = 1 << 0;
		const NATIVE   = 1 << 1;
		const UI       = 1 << 2;
		const PLAY     = 1 << 3;
	}
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum ClassInnerKind {
	FunctionDeclaration(FunctionDeclaration),
	MemberDeclaration(MemberDeclaration),
	Enum(EnumDefinition),
	Struct(StructDefinition),
	Const(ConstDefinition),
	Property(PropertyDefinition),
	Flag(FlagDefinition),
	StaticConstArray(StaticConstArray),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ClassInner {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ClassInnerKind,
}

impl ClassInner {
	pub fn name(&self) -> &Identifier {
		match &self.kind {
			ClassInnerKind::FunctionDeclaration(x) => &x.name,
			ClassInnerKind::MemberDeclaration(x) => &x.name,
			ClassInnerKind::Enum(x) => &x.name,
			ClassInnerKind::Struct(x) => &x.name,
			ClassInnerKind::Const(x) => &x.name,
			ClassInnerKind::Property(x) => &x.name,
			ClassInnerKind::StaticConstArray(x) => &x.name,
			ClassInnerKind::Flag(x) => &x.flag_name,
		}
	}
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ClassDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub ancestor: Option<Identifier>,
	pub flags: ClassDefinitionFlags,
	pub states: Vec<StatesItem>,
	pub defaults: Vec<DefaultStatement>,
	pub version: Option<VersionInfo>,
	pub replaces: Option<DottableId>,
	pub inners: HashMap<NameSymbol, Vec1<ClassInner>>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MixinClassDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub states: Vec<StatesItem>,
	pub defaults: Vec<DefaultStatement>,
	pub inners: HashMap<NameSymbol, Vec1<ClassInner>>,
}

bitflags! {
	#[derive(Serialize)]
	pub struct StructDefinitionFlags: u8 {
		const CLEAR_SCOPE = 1 << 0;
		const ABSTRACT    = 1 << 1;
		const NATIVE      = 1 << 2;
		const UI          = 1 << 3;
		const PLAY        = 1 << 4;
	}
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum StructInnerKind {
	FunctionDeclaration(FunctionDeclaration),
	MemberDeclaration(MemberDeclaration),
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

impl StructInner {
	pub fn name(&self) -> &Identifier {
		match &self.kind {
			StructInnerKind::FunctionDeclaration(x) => &x.name,
			StructInnerKind::MemberDeclaration(x) => &x.name,
			StructInnerKind::Enum(x) => &x.name,
			StructInnerKind::Const(x) => &x.name,
			StructInnerKind::StaticConstArray(x) => &x.name,
		}
	}
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StructDefinition {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub flags: StructDefinitionFlags,
	pub version: Option<VersionInfo>,
	pub inners: HashMap<NameSymbol, Vec1<StructInner>>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StaticConstArray {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub arr_type: Type,
	pub name: Identifier,
	pub exprs: ExprList,
}

bitflags! {
	#[derive(Serialize)]
	pub struct FunctionFlags: u16 {
		const NATIVE        = 1 << 0;
		const STATIC        = 1 << 1;
		const PRIVATE       = 1 << 2;
		const PROTECTED     = 1 << 3;
		const FINAL         = 1 << 4;
		const TRANSIENT     = 1 << 5;
		const VIRTUAL       = 1 << 6;
		const OVERRIDE      = 1 << 7;
		const ABSTRACT      = 1 << 8;
		const VAR_ARG       = 1 << 9;
		const UI            = 1 << 10;
		const PLAY          = 1 << 11;
		const CLEAR_SCOPE   = 1 << 12;
		const VIRTUAL_SCOPE = 1 << 13;
	}
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub struct Deprecated {
	pub version: VersionInfo,
	pub message: Option<StringConst>,
}

bitflags! {
	#[derive(Serialize)]
	pub struct ActionFlags: u8 {
		const ACTOR   = 1 << 0;
		const OVERLAY = 1 << 1;
		const WEAPON  = 1 << 2;
		const ITEM    = 1 << 3;
	}
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
	SingleUserType(Identifier),
	DottedUserType(DottableId),
	NativeType(Identifier),
	ReadonlyType(Identifier),
	ReadonlyNativeType(Identifier),
	Class(Option<DottableId>),
	Map(Box<(Type, Type)>),
	Array(Box<Type>, Option<Expression>),
	DynArray(Box<Type>),
	Let,
	Error,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeListOrVoidKind {
	TypeList(Vec1<Type>),
	Void,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TypeListOrVoid {
	pub span: Span,
	#[serde(flatten)]
	pub kind: TypeListOrVoidKind,
}

bitflags! {
	#[derive(Serialize)]
	pub struct FuncParamFlags: u8 {
		const IN       = 1 << 0;
		const OUT      = 1 << 1;
		const OPTIONAL = 1 << 2;
	}
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FuncParam {
	pub span: Span,
	pub flags: FuncParamFlags,
	pub param_type: Type,
	pub name: Identifier,
	pub init: Option<Expression>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FuncParams {
	pub span: Span,
	pub args: Vec<FuncParam>,
	pub variadic: bool,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub constant: bool,
	pub flags: FunctionFlags,
	pub deprecated: Option<Deprecated>,
	pub version: VersionInfo,
	pub action: Option<ActionFlags>,
	pub return_types: TypeListOrVoid,
	pub params: FuncParams,
	pub body: Option<CompoundStatement>,
}

bitflags! {
	#[derive(Serialize)]
	pub struct MemberFlags: u16 {
		const NATIVE      = 1 << 0;
		const PRIVATE     = 1 << 1;
		const PROTECTED   = 1 << 2;
		const TRANSIENT   = 1 << 3;
		const READ_ONLY   = 1 << 4;
		const INTERNAL    = 1 << 5;
		const VAR_ARG     = 1 << 6;
		const UI          = 1 << 7;
		const PLAY        = 1 << 8;

		// only allowed in classes, not structs
		const META        = 1 << 9;

		// only allowed within the base archive
		const CLEAR_SCOPE = 1 << 10;
	}
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MemberDeclaration {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub flags: MemberFlags,
	pub deprecated: Option<Deprecated>,
	pub version: VersionInfo,
	pub member_type: Type,
	pub name: Identifier,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct CompoundStatement {
	pub span: Option<Span>,
	pub statements: Vec<Statement>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
	Labeled(LabeledStatement),
	Compound(CompoundStatement),
	Expression(Expression),
	If {
		cond: Expression,
		body: CompoundStatement,
		else_body: Option<CompoundStatement>,
	},
	Switch {
		val: Expression,
		body: CompoundStatement,
	},
	Loop(CompoundStatement),
	Break,
	Continue,
	Return(Option<ExprList>),
	LocalVariableDefinition(LocalVariableDefinition),
	MultiAssign {
		assignees: ExprList,
		rhs: Expression,
	},
	StaticConstArray(StaticConstArray),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Statement {
	pub span: Option<Span>,
	#[serde(flatten)]
	pub kind: StatementKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum VarInit {
	Single(Expression),
	Compound(ExprList),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LocalVariableDefinition {
	pub span: Span,
	pub var_type: Type,
	pub name: Identifier,
	pub init: Option<VarInit>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
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

bitflags! {
	#[derive(Serialize)]
	pub struct StateLineFlags: u8 {
		const BRIGHT    = 1 << 1;
		const FAST      = 1 << 2;
		const SLOW      = 1 << 3;
		const NO_DELAY  = 1 << 4;
		const CAN_RAISE = 1 << 5;
	}
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StateLine {
	pub span: Span,
	pub sprite: NonWhitespace,
	pub frames: NonWhitespace,
	pub duration: Expression,
	pub flags: StateLineFlags,
	pub action_flags: Option<ActionFlags>,
	pub offset: Option<(Expression, Expression)>,
	pub light: Option<Vec1<StringConst>>,
	pub action: Option<StateLineAction>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
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
#[serde(tag = "kind", content = "data")]
#[derive(Debug, Clone, PartialEq)]
pub enum StatesItemKind {
	Label(NonWhitespace),
	Line(Box<StateLine>),
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
pub struct StatesItem {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StatesItemKind,
}
