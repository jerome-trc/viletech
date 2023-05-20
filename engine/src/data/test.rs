use std::path::PathBuf;

use rayon::prelude::*;

use super::*;

#[must_use]
fn request() -> LoadRequest<PathBuf, &'static str> {
	let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("..")
		.join("sample");

	let load_order = vec![
		(base.join("freedoom1.wad"), "/freedoom1"),
		(base.join("freedoom2.wad"), "/freedoom2"),
	];

	for (real_path, _) in &load_order {
		if !real_path.exists() {
			panic!(
				"Load/unload testing depends on the following files of sample data:\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom1.wad`\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom2.wad`\r\n\t\
				They can be acquired from https://freedoom.github.io/."
			);
		}
	}

	LoadRequest {
		load_order,
		tracker: None,
		dev_mode: false,
	}
}

#[test]
fn load() {
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
			panic!("Unexpected load outcome: {other:#?}");
		}
	}

	assert_eq!(
		catalog.mounts().len(),
		2,
		"Expected 2 mounts, found: {}",
		catalog.mounts().len()
	);

	// Root, 2 mounts, freedoom1.wad's contents, freedoom2.wad's contents.
	const EXPECTED: usize = 1 + 2 + 3081 + 3649;

	assert_eq!(
		catalog.vfs.files_len(),
		EXPECTED,
		"Expected {EXPECTED} mounted files, found: {}",
		catalog.vfs.files_len()
	);
}

#[test]
fn vfs_lookup() {
	let mut catalog = Catalog::new([]);
	let _ = catalog.load(request());

	assert!(catalog.vfs().get("/").is_some(), "Root lookup failed.");
	assert!(catalog.vfs().get("//").is_some(), "`//` lookup failed."); // Should return root.

	assert!(
		catalog.vfs().get("/freedoom2").is_some(),
		"`/freedoom2` lookup failed."
	);
	assert!(
		catalog.vfs().get("/freedoom2/").is_some(),
		"`/freedoom2/` lookup failed."
	);
	assert!(
		catalog.vfs().get("/freedoom2/FCGRATE2").is_some(),
		"`/freedoom2/FCGRATE2` lookup failed."
	);
	assert!(
		catalog.vfs().get("/freedoom2/FCGRATE2").is_some(),
		"`/freedoom2/FCGRATE2` lookup failed."
	);
	assert!(
		catalog.vfs().get("/freedoom2/FCGRATE2/").is_some(),
		"`/freedoom2/FCGRATE2/` lookup failed."
	);
}

#[test]
fn vfs_dir_structure() {
	let mut catalog = Catalog::new([]);
	let _ = catalog.load(request());

	let root = catalog.vfs().get("/").unwrap();

	assert_eq!(
		root.child_count(),
		2,
		"Expected root to have 2 children, but it has {}.",
		root.child_count()
	);

	const EXPECTED_CHILDREN: &[&str] = &[
		"/freedoom2/MAP01/THINGS",
		"/freedoom2/MAP01/LINEDEFS",
		"/freedoom2/MAP01/SIDEDEFS",
		"/freedoom2/MAP01/VERTEXES",
		"/freedoom2/MAP01/SEGS",
		"/freedoom2/MAP01/SSECTORS",
		"/freedoom2/MAP01/NODES",
		"/freedoom2/MAP01/SECTORS",
		"/freedoom2/MAP01/REJECT",
		"/freedoom2/MAP01/BLOCKMAP",
	];

	for (index, child) in catalog
		.vfs
		.get("/freedoom2/MAP01")
		.expect("`/freedoom2/MAP01` was not found.")
		.children()
		.expect("`/freedoom2/MAP01` is not a directory.")
		.enumerate()
	{
		assert_eq!(child.path_str(), EXPECTED_CHILDREN[index]);
	}
}

#[test]
fn glob() {
	let mut catalog = Catalog::new([]);
	let _ = catalog.load(request());

	{
		let glob = globset::Glob::new("/freedoom2/FCGRATE*").unwrap();
		let count = catalog.vfs.glob_par(glob).count();
		assert_eq!(
			count, 2,
			"Expected 2 entries matching glob `/freedoom2/FCGRATE*`, found: {}",
			count
		);
	}

	{
		let glob = globset::Glob::new("/freedoom1/E*M[0123456789]").unwrap();
		let count = catalog.vfs.glob_par(glob).count();
		assert_eq!(count, 36, "Expected 36 maps, found: {}", count);
	}

	{
		let glob = globset::Glob::new("/freedoom2/MAP*").unwrap();
		let count = catalog.vfs.glob_par(glob).count();
		assert_eq!(
			count, 352,
			"Expected 352 maps and sub-entries, found: {}",
			count
		);
	}
}
