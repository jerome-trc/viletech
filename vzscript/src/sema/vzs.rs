//! Semantic mid-section for VZScript.

use crate::{
	compile::symbol::{DefIx, Symbol},
	SyntaxNode,
};

use super::SemaContext;

#[must_use]
pub(super) fn define(_: &SemaContext, _: &SyntaxNode, _: &Symbol) -> DefIx {
	unimplemented!("...soon!")
}
