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

use crate::utils::lang::FileSpan;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Literal<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: LiteralKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum LiteralKind<'inp> {
	String(StringLiteral<'inp>),
	Int(IntLiteral<'inp>),
	Float(FloatLiteral<'inp>),
	Bool(bool),
	Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StringLiteral<'inp> {
	pub span: FileSpan<'inp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IntLiteral<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: IntLiteralKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum IntLiteralKind {
	I8(i8),
	U8(u8),
	I16(i16),
	U16(u16),
	I32(i32),
	U32(u32),
	I64(i64),
	U64(u64),
	I128(i128),
	U128(u128),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FloatLiteral<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: FloatLiteralKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum FloatLiteralKind {
	F32(f32),
	F64(f64),
}
