//! End-to-end testing of the compilation pipeline.

use std::path::Path;

use super::*;

#[test]
fn end_to_end() {
	// Improve clarity of panic messages.
	rayon::ThreadPoolBuilder::new()
		.thread_name(|i| format!("lith_global{i}"))
		.num_threads(1)
		.build_global()
		.unwrap();

	let core_path = Path::new(env!("CARGO_WORKSPACE_DIR")).join("assets/viletech/lith");

	let corelib = LibMeta {
		name: "lith".to_string(),
		version: Version::V0_0_0,
		native: true,
	};

	let mut compiler = Compiler::new(Config {
		opt: OptLevel::None,
		hotswap: false,
	});

	let reg_result = compiler.register_lib(corelib, |ftree| ftree.add_from_fs(&core_path));

	if let Err(errs) = reg_result {
		for err in errs {
			dbg!(err);
		}

		panic!();
	}

	compiler.finish_registration();

	crate::declare_symbols(&mut compiler);

	if compiler.any_errors() {
		for issue in compiler.drain_issues() {
			dbg!(issue);
		}

		panic!();
	}

	crate::resolve_imports(&mut compiler);

	if compiler.any_errors() {
		for issue in compiler.drain_issues() {
			dbg!(issue);
		}

		panic!();
	}

	crate::semantic_check(&mut compiler);

	if compiler.any_errors() {
		for issue in compiler.drain_issues() {
			dbg!(issue);
		}

		panic!();
	}

	let _ = crate::finalize(compiler, true, true);
}
