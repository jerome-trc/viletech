use serde::Serialize;

use crate::utils::lang::{Identifier, Span};

use super::{class::ClassInnerKind, item::Item, VariableDeclaration};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MixinClassDef {
	pub span: Span,
	pub name: Identifier,
	pub inners: Vec<MixinClassInner>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MixinClassInner {
	pub span: Span,
	#[serde(flatten)]
	pub kind: MixinClassInnerKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MixinClassInnerKind {
	Field(VariableDeclaration),
	Item(Item),
}

impl MixinClassInnerKind {
	pub(crate) fn map_to_class_inner_kind(self) -> ClassInnerKind {
		match self {
			MixinClassInnerKind::Field(field) => ClassInnerKind::Field(field),
			MixinClassInnerKind::Item(item) => ClassInnerKind::Item(item),
		}
	}
}
