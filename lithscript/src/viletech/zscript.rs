//! Transpilation, semantic checking, and LIR lowering for GZDoom's ZScript.

use super::{GlobalExportTable, LibSource};

pub(super) fn export(source: &LibSource, _: &GlobalExportTable) {
	let Some(inctree) = &source.zscript else { return; };
	assert!(!inctree.any_errors())
}
