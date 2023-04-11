use std::path::PathBuf;

use super::*;

#[must_use]
fn request() -> LoadRequest<PathBuf, &'static str> {
	let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("..")
		.join("sample");

	let paths = vec![
		(base.join("freedoom1.wad"), "freedoom1"),
		(base.join("freedoom2.wad"), "/freedoom2"),
	];

	for (real_path, _) in &paths {
		if !real_path.exists() {
			panic!(
				"VFS benchmarking depends on the following files of sample data:\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom1.wad`\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom2.wad`\r\n\t\
				They can be acquired from https://freedoom.github.io/."
			);
		}
	}

	LoadRequest {
		paths,
		tracker: None,
	}
}

#[test]
fn vfs_path_hash() {
	let k_root = VfsKey::new("/");
	let k_empty = VfsKey::new("");
	assert!(k_root == k_empty);
}

#[test]
fn load() {
	let mut catalog = Catalog::default();
	let outcome = catalog.load(request());

	match outcome {
		LoadOutcome::Cancelled => {
			panic!("Unexpected mount cancellation.");
		}
		LoadOutcome::MountFail { .. } => {
			panic!("Mount failure.")
		}
		LoadOutcome::PostProcFail { .. } => {
			panic!("Post-processing failure.")
		}
		LoadOutcome::Ok { mount, pproc } => {
			assert_eq!(mount.len(), 2);
			assert_eq!(pproc.len(), 2);

			assert!(
				mount[0].is_empty()
					&& pproc[0].is_empty()
					&& mount[1].is_empty()
					&& pproc[1].is_empty()
			);
		}
	}

	assert!(
		catalog.mounts().len() == 2,
		"Expected 2 mounts, found: {len}",
		len = catalog.mounts().len()
	);

	// Root, 2 mounts, freedoom1.wad's contents, freedoom2.wad's contents.
	const EXPECTED: usize = 1 + 2 + 3081 + 3649;

	assert!(
		catalog.all_files().count() == EXPECTED,
		"Expected {EXPECTED} mounted files, found: {count}",
		count = catalog.all_files().count()
	);
}

#[test]
fn vfs_lookup() {
	let mut catalog = Catalog::default();
	let _ = catalog.load(request());

	assert!(catalog.get_file("/").is_some(), "Root lookup failed.");
	assert!(catalog.get_file("//").is_some(), "`//` lookup failed."); // Should return root.

	assert!(
		catalog.get_file("/freedoom2").is_some(),
		"`/freedoom2` lookup failed."
	);
	assert!(
		catalog.get_file("/freedoom2/").is_some(),
		"`/freedoom2/` lookup failed."
	);
	assert!(
		catalog.get_file("freedoom2/FCGRATE2").is_some(),
		"`freedoom2/FCGRATE2` lookup failed."
	);
	assert!(
		catalog.get_file("/freedoom2/FCGRATE2").is_some(),
		"`/freedoom2/FCGRATE2` lookup failed."
	);
	assert!(
		catalog.get_file("/freedoom2/FCGRATE2/").is_some(),
		"`/freedoom2/FCGRATE2/` lookup failed."
	);
}

#[test]
fn vfs_dir_structure() {
	let mut catalog = Catalog::default();
	let _ = catalog.load(request());

	let root = catalog.get_file("/").unwrap();

	assert_eq!(
		root.children().count(),
		2,
		"Expected root to have 2 children, but it has {}.",
		root.children().count()
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
		.get_file("/freedoom2/MAP01")
		.unwrap()
		.children()
		.enumerate()
	{
		assert_eq!(child.path_str(), EXPECTED_CHILDREN[index]);
	}
}

#[test]
fn glob() {
	let mut catalog = Catalog::default();
	let _ = catalog.load(request());
	let glob = globset::Glob::new("/freedoom2/FCGRATE*").unwrap();
	let count = catalog.get_files_glob_par(glob).count();
	assert!(
		count == 2,
		"Expected 2 entries matching glob `/freedoom2/FCGRATE*`, found: {}",
		count
	);

	let glob = globset::Glob::new("/freedoom1/E*M[0123456789]").unwrap();
	let count = catalog.get_files_glob_par(glob).count();
	assert!(count == 36, "Expected 36 maps, found: {}", count);

	let glob = globset::Glob::new("/freedoom2/MAP*").unwrap();
	let count = catalog.get_files_glob_par(glob).count();
	assert!(count == 32, "Expected 32 maps, found: {}", count);
}
