use std::path::PathBuf;

use super::*;

#[must_use]
fn sample_paths() -> [(PathBuf, &'static str); 2] {
	let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("..")
		.join("sample");

	let ret = [
		(base.join("freedoom1.wad"), "freedoom1"),
		(base.join("freedoom2.wad"), "/freedoom2"),
	];

	for (real_path, _) in &ret {
		if !real_path.exists() {
			panic!(
				"VFS benchmarking depends on the following files of sample data:\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom1.wad`\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom2.wad`\r\n\t\
				They can be acquired from https://freedoom.github.io/."
			);
		}
	}

	ret
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
	let results = catalog.load_simple(&sample_paths());

	assert!(
		results.len() == 2,
		"Expected 2 results, got: {len}",
		len = results.len()
	);

	assert!(
		results[0].is_ok(),
		"Failed to mount `freedoom1.wad`: {res}",
		res = results[0].as_ref().unwrap_err()
	);

	assert!(
		results[1].is_ok(),
		"Failed to mount `freedoom2.wad`: {res}",
		res = results[1].as_ref().unwrap_err()
	);

	assert!(
		catalog.mounts().len() == 2,
		"Expected 2 mounts, found: {len}",
		len = catalog.mounts().len()
	);

	// Root, 2 mounts, freedoom1.wad's contents, freedoom2.wad's contents
	const EXPECTED: usize = 1 + 2 + 3081 + 3649;

	assert!(
		catalog.all_files().count() == EXPECTED,
		"Expected {EXPECTED} mounts, found: {count}",
		count = catalog.all_files().count()
	);
}

#[test]
fn vfs_lookup() {
	let mut catalog = Catalog::default();
	let _ = catalog.load_simple(&sample_paths());

	assert!(catalog.get_file("/").is_some(), "Root lookup failed.");
	assert!(catalog.get_file("//").is_some(), "`//` lookup failed."); // Should return root

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
	let _ = catalog.load_simple(&sample_paths());

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
	let _ = catalog.load_simple(&sample_paths());
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
