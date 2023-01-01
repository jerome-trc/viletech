use serde::Serialize;

use crate::utils::lang::{Identifier, Span};

use super::{item::Item, Resolver, VariableDeclaration};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassDef {
	pub span: Span,
	pub name: Identifier,
	pub ancestors: Vec<Resolver>,
	pub quals: Vec<ClassQualifier>,
	pub inners: Vec<ClassInner>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassExtend {
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<ClassInner>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassInner {
	pub span: Span,
	#[serde(flatten)]
	pub kind: ClassInnerKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum ClassInnerKind {
	Mixin(Identifier),
	Field(VariableDeclaration),
	Item(Item),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassQualifier {
	pub span: Span,
	pub kind: ClassQualKind,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum ClassQualKind {
	Abstract,
	Final,
}
