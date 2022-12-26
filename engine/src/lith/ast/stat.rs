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
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use serde::Serialize;

use crate::utils::lang::{Identifier, Span};

use super::{expr::Expression, item::Item, Annotation, BlockLabel, TypeExpr};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Statement {
	pub span: Span,
	#[serde(flatten)]
	pub kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "def")]
pub enum StatementKind {
	Empty,
	Break {
		target: Option<String>,
	},
	Continue {
		target: Option<String>,
	},
	Item(Item),
	Binding(Binding),
	Expression(Expression),
	Block(StatementBlock),
	If {
		cond: Expression,
		body: StatementBlock,
		else_body: Option<Box<Statement>>,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
	Switch {
		val: Expression,
		label: Option<BlockLabel>,
		cases: Vec<SwitchCase>,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
	Loop {
		kind: LoopKind,
		body: Box<StatementBlock>,
		/// Outer annotations only.
		annotations: Vec<Annotation>,
	},
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatementBlock {
	pub span: Span,
	pub label: Option<BlockLabel>,
	pub statements: Vec<Statement>,
	/// Inner annotations only, applied to the entire block.
	pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Binding {
	pub span: Span,
	pub names: Vec<Identifier>,
	pub init: Option<Expression>,
	pub type_spec: Option<TypeExpr>,
	pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SwitchCase {
	pub span: Span,
	pub kind: SwitchCaseKind,
	pub block: StatementBlock,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum SwitchCaseKind {
	Default,
	Specific(Expression),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum LoopKind {
	Infinite,
	Range {
		bindings: Vec<Identifier>,
		sequence: Expression,
	},
	While {
		condition: Expression,
	},
	DoWhile {
		condition: Expression,
	},
	DoUntil {
		condition: Expression,
	},
}
