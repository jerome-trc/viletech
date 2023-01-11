//! Symbols representing a LithScript abstract syntax tree.

mod expr;
mod item;
mod literal;
mod stat;

use serde::Serialize;
use vec1::Vec1;

use crate::utils::lang::{Identifier, Span};

pub use expr::*;
pub use item::*;
pub use literal::*;
pub use stat::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModuleTree {
	pub top_level: Vec<TopLevel>,
}

impl ModuleTree {
	pub fn includes(&self) -> impl Iterator<Item = &StringLiteral> {
		self.top_level.iter().filter_map(|top| {
			let directive = match top {
				TopLevel::Preproc(directive) => directive,
				_ => {
					return None;
				}
			};

			match &directive.kind {
				PreprocDirectiveKind::Include(inc) => Some(inc),
				_ => None,
			}
		})
	}
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TopLevel {
	/// Inner annotations only, applied to the entire translation unit.
	Annotation(Annotation),
	Item(Item),
	Preproc(PreprocDirective),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PreprocDirective {
	pub span: Span,
	pub kind: PreprocDirectiveKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PreprocDirectiveKind {
	Edition(StringLiteral),
	Include(StringLiteral),
	Namespace(Identifier),
}

/// A "resolver" is a double-colon-separated token chain, named after the
/// concept of "scope resolution". These are the Lith counterpart to Rust "paths",
/// named differently to disambiguate from the filesystem idea of a "path". A
/// single identifier can be a valid resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resolver {
	pub span: Span,
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
	pub args: Vec<CallArg>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BlockLabel {
	pub span: Span,
	pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypeExpr {
	pub span: Span,
	#[serde(flatten)]
	pub kind: TypeExprKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum TypeExprKind {
	Void,
	Primitive(PrimitiveTypeKind),
	Resolver(Resolver),
	Array(Box<ArrayTypeExpr>),
	Tuple { members: Vec<TypeExpr> },
	Inferred,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArrayTypeExpr {
	pub storage: TypeExpr,
	pub length: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PrimitiveTypeKind {
	Bool,
	Char,
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
	F32,
	F64,
}
