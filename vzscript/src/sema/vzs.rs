//! Semantic mid-section for VZScript.

use crate::{
	compile::symbol::{DefIx, Symbol},
	SyntaxNode,
};

use super::SemaContext;

#[must_use]
pub(super) fn define(ctx: &SemaContext, root: &SyntaxNode, symbol: &Symbol) -> DefIx {
	let ast = root
		.covering_element(ctx.location.span)
		.into_node()
		.unwrap();

	unimplemented!("...soon!")
}
