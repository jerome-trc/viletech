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

use crate::utils::lang::{Span, Identifier};

use super::{
	expr::{ExprList, Expression, TypeExpr},
	item::ItemDef,
	Annotation, BlockLabel,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Statement {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "def")]
pub enum StatementKind {
	Expression(Expression),
	Compound(CompoundStatement),
	If {
		cond: Expression,
		body: CompoundStatement,
		else_body: Option<CompoundStatement>,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
	Switch {
		val: Expression,
		label: Option<BlockLabel>,
		body: Vec<CompoundStatement>,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
	CondIter {
		cond: Expression,
		kind: CondIterKind,
		body: Box<CompoundStatement>,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
	ForIter {
		init: Option<ForInit>,
		cond: Option<Expression>,
		update: Option<ExprList>,
		body: Box<CompoundStatement>,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
	ForEachIter {
		var_name: Identifier,
		sequence: Expression,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
	Item(ItemDef),
	LocalVarDecl(LocalVarDecl),
	MultiAssign {
		assignees: ExprList,
		rhs: Expression,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
	Continue,
	Break,
	Empty,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompoundStatement {
	pub span: Span,
	pub label: Option<BlockLabel>,
	pub statements: Vec<Statement>,
	/// Inner annotations only, applied to the entire block.
	pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalVarDecl {
	pub span: Span,
	pub name: Identifier,
	pub type_spec: Option<TypeExpr>,
	pub init: Option<Expression>,
	pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum CondIterKind {
	Loop,
	While,
	DoWhile,
	DoUntil,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ForInit {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ForInitKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ForInitKind {
	VarDef(LocalVarDecl),
	ExprList(ExprList),
}
