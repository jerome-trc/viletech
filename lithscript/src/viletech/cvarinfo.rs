//! Transpilation, semantic checking, and LIR lowering for GZDoom's CVARINFO.

use super::{GlobalExportTable, LibSource};

pub(super) fn export(source: &LibSource, _: &GlobalExportTable) {
	for cvarinfo in &source.cvarinfo {
		assert!(cvarinfo.errors.is_empty());
		let _ = cvarinfo.cursor();
		// ???
	}
}
