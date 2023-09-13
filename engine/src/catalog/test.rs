use std::path::Path;

use super::*;

#[test]
fn load_unload() {
	let mut catalog = Catalog::new([]);
	let outcome = catalog.load(request());

	match outcome {
		LoadOutcome::Ok { mount, prep } => {
			assert_eq!(mount.len(), 2);
			assert_eq!(prep.len(), 2);

			assert!(
				mount[0].is_empty()
					&& prep[0].is_empty()
					&& mount[1].is_empty()
					&& prep[1].is_empty()
			);
		}
		other => {
			panic!("unexpected load outcome: {other:#?}");
		}
	}

	assert_eq!(
		catalog.vfs().mounts().len(),
		2,
		"expected 2 mounts, found: {}",
		catalog.vfs().mounts().len()
	);

	// Root, 2 mounts, freedoom1.wad's contents, freedoom2.wad's contents.
	const EXPECTED: usize = 1 + 2 + 3081 + 3649;

	assert_eq!(
		catalog.vfs.file_count(),
		EXPECTED,
		"expected {EXPECTED} mounted files, found: {}",
		catalog.vfs.file_count()
	);

	catalog.clear();
}

// Details /////////////////////////////////////////////////////////////////////

#[must_use]
fn request() -> LoadRequest {
	let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample");

	let load_order = vec![
		(base.join("freedoom1.wad"), VPathBuf::from("/freedoom1")),
		(base.join("freedoom2.wad"), VPathBuf::from("/freedoom2")),
	];

	for (real_path, _) in &load_order {
		if !real_path.exists() {
			panic!(
				"load/unload testing depends on the following files of sample data:\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom1.wad`\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom2.wad`\r\n\t\
				They can be acquired from https://freedoom.github.io/"
			);
		}
	}

	LoadRequest {
		mount: MountRequest {
			load_order,
			tracker: None,
			basedata: true,
		},
		tracker: None,
		dev_mode: false,
	}
}
