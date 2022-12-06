//! Statements of all kinds.

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

use super::{expr::{Expression, TypeExpr, ExprList}, item::ItemDef, BlockLabel};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Statement<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: StatementKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "def")]
pub enum StatementKind<'inp> {
	Expression(Expression<'inp>),
	Compound(CompoundStatement<'inp>),
	If {
		cond: Expression<'inp>,
		body: CompoundStatement<'inp>,
		else_body: Option<CompoundStatement<'inp>>,
	},
	Switch {
		val: Expression<'inp>,
		label: Option<BlockLabel<'inp>>,
		body: Vec<CompoundStatement<'inp>>,
	},
	CondIter {
		cond: Expression<'inp>,
		kind: CondIterKind,
		body: Box<CompoundStatement<'inp>>,
	},
	ForIter {
		init: Option<ForInit<'inp>>,
		cond: Option<Expression<'inp>>,
		update: Option<ExprList<'inp>>,
		body: Box<CompoundStatement<'inp>>,
	},
	ForEachIter {
		var_name: Identifier<'inp>,
		sequence: Expression<'inp>,
	},
	Item(ItemDef<'inp>),
	LocalVarDecl(LocalVarDecl<'inp>),
	MultiAssign {
		assignees: ExprList<'inp>,
		rhs: Expression<'inp>,
	},
	Continue,
	Break,
	Empty,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompoundStatement<'inp> {
	pub span: FileSpan<'inp>,
	pub label: Option<BlockLabel<'inp>>,
	pub statements: Vec<Statement<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVarDecl<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub type_spec: Option<TypeExpr<'inp>>,
	pub init: Option<Expression<'inp>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]

pub enum CondIterKind {
	Loop,
	While,
	DoWhile,
	DoUntil,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ForInit<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: ForInitKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ForInitKind<'inp> {
	VarDef(LocalVarDecl<'inp>),
	ExprList(ExprList<'inp>),
}
