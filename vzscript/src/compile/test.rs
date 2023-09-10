//! End-to-end compilation testing.

use std::path::Path;

use crate::{
	inctree::{ParsedFile, SourceKind},
	IncludeTree,
};

use super::*;

#[test]
fn smoke() {
	let corelib = LibSource {
		name: "vzscript".to_string(),
		version: crate::Version::new(0, 0, 0),
		native: true,
		inctree: IncludeTree::from_fs(
			&Path::new(env!("CARGO_WORKSPACE_DIR")).join("assets/viletech"),
			Path::new("vzscript/main.vzs"),
			Some(Version::new(0, 0, 0)),
		),
		decorate: None,
	};

	let userlib = LibSource {
		name: "my_mod".to_string(),
		version: crate::Version::new(0, 0, 0),
		native: false,
		inctree: IncludeTree {
			files: vec![],
			errors: vec![],
		},
		decorate: None,
	};

	if corelib.inctree.any_errors() || userlib.inctree.any_errors() {
		for err in &corelib.inctree.errors {
			dbg!(err);
		}

		for err in &userlib.inctree.errors {
			dbg!(err);
		}

		for pfile in &corelib.inctree.files {
			let SourceKind::Vzs(ptree) = pfile.inner() else { unreachable!() };

			if ptree.any_errors() {
				eprintln!("Parse errors in {}:", pfile.path());
			}

			for err in ptree.errors() {
				dbg!(err);
			}
		}

		for pfile in &userlib.inctree.files {
			match pfile.inner() {
				SourceKind::Vzs(ptree) => {
					if ptree.any_errors() {
						eprintln!("Parse errors in {}:", pfile.path());
					}

					for err in ptree.errors() {
						dbg!(err);
					}
				}
				SourceKind::Zs(ptree) => {
					if ptree.any_errors() {
						eprintln!("Parse errors in {}:", pfile.path());
					}

					for err in ptree.errors() {
						dbg!(err);
					}
				}
			}
		}

		panic!("failed to compose an include tree");
	}

	let mut compiler = Compiler::new([corelib, userlib]);

	crate::front::declare_symbols(&mut compiler);

	if compiler.any_errors() {
		for issue in compiler.drain_issues() {
			dbg!(issue);
		}

		panic!();
	}

	crate::sema::sema(&mut compiler);

	if compiler.any_errors() {
		for issue in compiler.drain_issues() {
			dbg!(issue);
		}

		panic!();
	}
}
