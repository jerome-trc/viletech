//! Semantic mid-section for VZScript.

use std::sync::Arc;

use crate::compile::symbol::Symbol;

use super::SemaContext;

#[must_use]
pub(super) fn define(_: &SemaContext, _: Arc<Symbol>) -> Option<Symbol> {
	unimplemented!("...soon!")
}
