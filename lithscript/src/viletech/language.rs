//! Transpilation, semantic checking, and LIR lowering for GZDoom's LANGUAGE.

use super::{GlobalExportTable, LibSource};

pub(super) fn export(source: &LibSource, _: &GlobalExportTable) {
	for lang in &source.language {
		assert!(lang.errors.is_empty());
		let _ = lang.cursor();
		// ???
	}
}
