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

use crate::utils::lang::{Span, StringHandle};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Literal {
	pub span: Span,
	#[serde(flatten)]
	pub kind: LiteralKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum LiteralKind {
	Null,
	Bool(bool),
	Int(IntLiteral),
	Float(FloatLiteral),
	Char(CharLiteral),
	String(StringLiteral),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IntLiteral {
	pub span: Span,
	pub value: u128,
	#[serde(flatten)]
	pub type_spec: Option<IntType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum IntType {
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	I128,
	U128,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FloatLiteral {
	pub span: Span,
	pub value: f64,
	#[serde(flatten)]
	pub type_spec: Option<FloatType>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FloatType {
	F32,
	F64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CharLiteral {
	pub span: Span,
	pub character: char,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StringLiteral {
	pub span: Span,
	pub string: StringHandle,
}
