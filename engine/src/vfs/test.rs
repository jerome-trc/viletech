use std::path::PathBuf;

use super::VirtualFs;

#[must_use]
fn sample_paths() -> [(PathBuf, &'static str); 2] {
	let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("..")
		.join("sample");

	let ret = [
		(base.join("freedoom1.wad"), "freedoom1"),
		(base.join("freedoom2.wad"), "freedoom2"),
	];

	for tuple in &ret {
		if !tuple.0.exists() {
			panic!("VFS unit testing depends on sample/freedoom1.wad and sample/freedoom2.wad.");
		}
	}

	ret
}

#[test]
fn mount() {
	let mut vfs = VirtualFs::default();
	let results = vfs.mount(&sample_paths());
	assert!(results.len() == 2);
	assert!(results[0].is_ok() && results[1].is_ok());
	assert!(vfs.mount_count() == 2);
}

#[test]
fn lookup() {
	let mut vfs = VirtualFs::default();
	let _ = vfs.mount(&sample_paths());

	assert!(vfs.lookup("/").is_some(), "Root lookup failed.");
	assert!(vfs.lookup("//").is_some(), "`//` lookup failed."); // Should return root

	assert!(
		vfs.lookup("/freedoom2").is_some(),
		"`/freedoom2` lookup failed."
	);
	assert!(
		vfs.lookup("/freedoom2/").is_some(),
		"`/freedoom2/` lookup failed."
	);
	assert!(
		vfs.lookup("freedoom2/FCGRATE2").is_some(),
		"`freedoom2/FCGRATE2` lookup failed."
	);
	assert!(
		vfs.lookup("/freedoom2/FCGRATE2").is_some(),
		"`/freedoom2/FCGRATE2` lookup failed."
	);
	assert!(
		vfs.lookup("/freedoom2/FCGRATE2/").is_some(),
		"`/freedoom2/FCGRATE2/` lookup failed."
	);
}

#[test]
fn dir_structure() {
	let mut vfs = VirtualFs::default();
	let _ = vfs.mount(&sample_paths());

	let root = vfs.root();

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

	for (index, child) in vfs
		.lookup("/freedoom2/MAP01")
		.unwrap()
		.children()
		.enumerate()
	{
		assert_eq!(child.path_str(), EXPECTED_CHILDREN[index]);
	}
}

#[test]
fn glob() {
	let mut vfs = VirtualFs::default();
	let _ = vfs.mount(&sample_paths());
	let glob = globset::Glob::new("/freedoom2/FCGRATE*").unwrap();
	let count = vfs.glob(glob).count();
	assert!(
		count == 2,
		"Expected 2 entries matching glob `/freedoom2/FCGRATE*`, found: {}",
		count
	);

	let glob = globset::Glob::new("/freedoom1/E*M*").unwrap();
	let count = vfs.glob(glob).count();
	assert!(count == 37, "Expected 37 maps, found: {}", count);

	let glob = globset::Glob::new("/freedoom2/MAP*").unwrap();
	let count = vfs.glob(glob).count();
	assert!(count == 32, "Expected 32 maps, found: {}", count);
}

#[test]
fn unmount() {
	let mut vfs = VirtualFs::default();
	let _ = vfs.mount(&sample_paths());

	vfs.unmount("/freedoom2").unwrap();
	assert_eq!(vfs.root().child_count(), 1);
	assert_eq!(vfs.mount_count(), 1);

	const EXPECTED_CHILDREN: &[&str] = &[
		"/freedoom1/E1M1/THINGS",
		"/freedoom1/E1M1/LINEDEFS",
		"/freedoom1/E1M1/SIDEDEFS",
		"/freedoom1/E1M1/VERTEXES",
		"/freedoom1/E1M1/SEGS",
		"/freedoom1/E1M1/SSECTORS",
		"/freedoom1/E1M1/NODES",
		"/freedoom1/E1M1/SECTORS",
		"/freedoom1/E1M1/REJECT",
		"/freedoom1/E1M1/BLOCKMAP",
	];

	for (index, child) in vfs
		.lookup("/freedoom1/E1M1")
		.unwrap()
		.children()
		.enumerate()
	{
		assert_eq!(child.path_str(), EXPECTED_CHILDREN[index]);
	}

	vfs.unmount("/freedoom1").unwrap();
	assert_eq!(vfs.root().child_count(), 0);
	assert_eq!(vfs.mount_count(), 0);
}
