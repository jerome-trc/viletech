//! Transpilation, semantic checking, and LIR lowering for GZDoom's DECORATE.

use super::{GlobalExportTable, LibSource};

pub(super) fn export(source: &LibSource, _: &GlobalExportTable) {
	let Some(inctree) = &source.decorate else { return; };
	assert!(!inctree.any_errors());
}
