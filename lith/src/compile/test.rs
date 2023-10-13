//! End-to-end testing of the compilation pipeline.

use std::path::Path;

use super::*;

#[test]
fn end_to_end() {
	let core_path = Path::new(env!("CARGO_WORKSPACE_DIR")).join("assets/viletech/lith");

	let ftree = FileTree::from_fs(&core_path).unwrap();

	if !ftree.valid() {
		for (path, ptree) in ftree.files() {
			eprintln!("Errors while parsing {path}:");

			for err in ptree.errors() {
				dbg!(err);
				eprintln!();
			}
		}

		panic!();
	}

	let corelib_src = LibSource {
		name: "lithica".to_string(),
		version: Version::new(0, 0, 0),
		native: true,
		filetree: ftree,
	};

	let mut compiler = Compiler::new(
		Config {
			opt: OptLevel::None,
		},
		[corelib_src],
	);

	crate::declare_symbols(&mut compiler);

	if compiler.any_errors() {
		for issue in compiler.drain_issues() {
			dbg!(issue);
		}

		panic!();
	}
}
