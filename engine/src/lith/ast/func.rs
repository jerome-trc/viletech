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

use super::{expr::Expression, decl::DeclQualifier};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionDecl<'inp> {
	pub span: FileSpan<'inp>,
	pub name: Identifier<'inp>,
	pub qualifiers: DeclQualifier<'inp>,
	pub body: Option<()>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FunctionCallArg<'inp> {
	pub span: FileSpan<'inp>,
	#[serde(flatten)]
	pub kind: FunctionCallArgKind<'inp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum FunctionCallArgKind<'inp> {
	Unnamed(Expression<'inp>),
	Named(Identifier<'inp>, Expression<'inp>),
}
