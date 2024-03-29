//! End-to-end testing of the compilation pipeline.

use super::*;

#[test]
fn end_to_end() {
	// Improve clarity of panic messages.
	rayon::ThreadPoolBuilder::new()
		.thread_name(|i| format!("lith_global{i}"))
		.num_threads(1)
		.build_global()
		.unwrap();

	let mut compiler = Compiler::new(Config {
		opt: OptLevel::None,
		hotswap: false,
	});

	compiler.finish_registration();

	crate::compile::declare_symbols(&mut compiler);

	if compiler.any_errors() {
		for issue in compiler.drain_issues() {
			dbg!(issue);
		}

		panic!();
	}

	crate::compile::semantic_check(&mut compiler);

	if compiler.any_errors() {
		for issue in compiler.drain_issues() {
			dbg!(issue);
		}

		panic!();
	}

	let _ = crate::compile::finalize(compiler, true, true);
}
