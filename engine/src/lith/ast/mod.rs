//! Symbols representing a LithScript abstract syntax tree.

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

mod class;
mod expr;
mod item;
mod literal;
mod mixin;
mod stat;

use serde::Serialize;
use vec1::Vec1;

use crate::utils::lang::{Span, Identifier};

pub use class::*;
pub use expr::*;
pub use item::*;
pub use literal::*;
pub use mixin::*;
pub use stat::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AbstractSyntaxTree {
	pub annotations: Vec<Annotation>,
	/// Inner annotations only, applied to the entire translation unit.
	pub items: Vec<ItemDef>,
}

/// A "resolver" is a double-colon-separated token chain, named after the
/// concept of "scope resolution". These are the Lith counterpart to Rust "paths",
/// named differently to disambiguate from the filesystem idea of a "path".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resolver {
	pub span: Span,
	pub outer: bool,
	pub parts: Vec1<ResolverPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolverPart {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ResolverPartKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ResolverPartKind {
	Identifier(Identifier),
	Super,
	SelfUppercase,
}

/// Equivalent to "attributes" in Rust and C#, and Java's feature of the same name.
/// These use the syntax `#[]` with an optional `!` in between the `#` and `[`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Annotation {
	pub span: Span,
	pub resolver: Resolver,
	/// If an exclamation mark is between the pound and left bracket, this is an
	/// "inner" annotation, and applies to the item/block it's declared in.
	/// Otherwise it's "outer" and applies to the next item/block.
	pub inner: bool,
	pub args: Vec<FunctionCallArg>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BlockLabel {
	pub span: Span,
}
