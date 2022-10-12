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

pub mod class;
pub mod decorate;
pub mod mixin;
pub mod states;
pub mod top;

use serde::Serialize;
use vec1::Vec1;

use super::{interner::StringSymbol, ir::*, Span};

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct StaticConstArray {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub arr_type: Type,
	pub name: Identifier,
	pub exprs: ExprList,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationMetadataItemKind {
	Native,
	Static,
	Private,
	Protected,
	Latent,
	Final,
	Meta,
	Transient,
	ReadOnly,
	Internal,
	Virtual,
	Override,
	Abstract,
	VarArg,
	UI,
	Play,
	ClearScope,
	VirtualScope,
	Deprecated {
		version: StringConst,
		message: Option<StringConst>,
	},
	Version(StringConst),
	Action(Option<Vec1<Identifier>>),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct DeclarationMetadataItem {
	pub span: Span,
	#[serde(flatten)]
	pub kind: DeclarationMetadataItemKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
	SingleUserType(Identifier),
	DottedUserType(DottableId),
	NativeType(Identifier),
	ReadonlyType(Identifier),
	ReadonlyNativeType(Identifier),
	Class(Option<DottableId>),
	Map(Box<(TypeOrArray, TypeOrArray)>),
	DynArray(Box<TypeOrArray>),
	Let,

	Error,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Type {
	pub span: Span,
	#[serde(flatten)]
	pub kind: TypeKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeOrArrayKind {
	Type(Type),
	Array(Type, ArraySizes),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TypeOrArray {
	pub span: Span,
	#[serde(flatten)]
	pub kind: TypeOrArrayKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeListOrVoidKind {
	TypeList(Vec1<TypeOrArray>),
	Void,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TypeListOrVoid {
	pub span: Span,
	#[serde(flatten)]
	pub kind: TypeListOrVoidKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum ParamFlagItemKind {
	In,
	Out,
	Optional,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ParamFlagItem {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ParamFlagItemKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FuncParam {
	pub span: Span,
	pub flags: Vec<ParamFlagItem>,
	pub param_type: Type,
	pub name: Identifier,
	pub init: Option<Expression>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum FuncParamsKind {
	Void,
	List {
		args: Vec<FuncParam>,
		variadic: bool,
	},
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FuncParams {
	pub span: Span,
	#[serde(flatten)]
	pub kind: FuncParamsKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub name: Identifier,
	pub constant: bool,
	pub metadata: Vec<DeclarationMetadataItem>,
	pub return_types: TypeListOrVoid,
	pub params: FuncParams,
	pub body: Option<CompoundStatement>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct MemberDeclaration {
	pub doc_comment: Option<StringSymbol>,
	pub span: Span,
	pub metadata: Vec<DeclarationMetadataItem>,
	pub member_type: TypeListOrVoid,
	pub vars: Vec1<(Identifier, Option<ArraySizes>)>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
	Function(FunctionDeclaration),
	Member(MemberDeclaration),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum CondIterType {
	While,
	Until,
	DoWhile,
	DoUntil,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum VarInitKind {
	Single {
		name: Identifier,
		val: Option<Expression>,
	},
	Array {
		name: Identifier,
		sizes: Option<ArraySizes>,
		vals: Option<ExprList>,
	},
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct VarInit {
	pub span: Span,
	#[serde(flatten)]
	pub kind: VarInitKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct LocalVariableDefinition {
	pub span: Span,
	pub var_type: Type,
	pub inits: Vec1<VarInit>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum ForInitKind {
	VarDef(LocalVariableDefinition),
	ExprList(ExprList),
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ForInit {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ForInitKind,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct CompoundStatement {
	pub span: Span,
	pub statements: Vec<Statement>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "def")]
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
	Labeled(LabeledStatement),
	Compound(CompoundStatement),
	Expression(Expression),
	If {
		cond: Expression,
		body: Box<Statement>,
		else_body: Option<Box<Statement>>,
	},
	Switch {
		val: Expression,
		body: Box<Statement>,
	},
	CondIter {
		cond: Expression,
		body: Box<Statement>,
		iter_type: CondIterType,
	},
	For {
		init: Option<ForInit>,
		cond: Option<Expression>,
		update: Option<ExprList>,
		body: Box<Statement>,
	},
	Break,
	Continue,
	Return(Option<ExprList>),
	LocalVariableDefinition(LocalVariableDefinition),
	MultiAssign {
		assignees: ExprList,
		rhs: Expression,
	},
	StaticConstArray(StaticConstArray),
	Empty,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Statement {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StatementKind,
}

// These types are kept in different files for clarity,
// but export them all as if the intermediate modules didn't exist

pub use class::*;
pub use decorate::*;
pub use mixin::*;
pub use states::*;
pub use top::*;
