use super::*;

#[test]
fn mount() {
	let Some(req) = request() else {
		return;
	};

	let mut vfs = VirtualFs::default();

	let outcome = vfs.mount(req);

	match outcome {
		MountOutcome::Ok(errors) => {
			assert_eq!(vfs.mounts.len(), 2);
			assert_eq!(errors.len(), 2);
			assert!(errors[0].is_empty());
		}
		other => {
			panic!("unexpected mount outcome: {other:#?}");
		}
	}
}

#[test]
fn lookup() {
	let Some(req) = request() else {
		return;
	};

	let mut vfs = VirtualFs::default();

	if vfs.mount(req).total_err_count() > 0 {
		panic!("VFS lookup unit test encountered mount errors");
	}

	assert!(vfs.get("/").is_some(), "root lookup failed");
	assert!(vfs.get("//").is_some(), "`//` lookup failed"); // Should return root.

	assert!(
		vfs.get("/freedoom2").is_some(),
		"`/freedoom2` lookup failed."
	);
	assert!(
		vfs.get("/freedoom2/").is_some(),
		"`/freedoom2/` lookup failed."
	);
	assert!(
		vfs.get("/freedoom2/FCGRATE2").is_some(),
		"`/freedoom2/FCGRATE2` lookup failed."
	);
	assert!(
		vfs.get("/freedoom2/FCGRATE2/").is_some(),
		"`/freedoom2/FCGRATE2/` lookup failed."
	);
}
#[test]
fn dir_structure() {
	let Some(req) = request() else {
		return;
	};

	let mut vfs = VirtualFs::default();

	if vfs.mount(req).total_err_count() > 0 {
		panic!("VFS directory structure unit test encountered mount errors");
	}

	let root = vfs.get("/").unwrap();

	assert_eq!(
		root.child_count(),
		2,
		"expected root to have 2 children, but it has {}",
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
		.get("/freedoom2/MAP01")
		.expect("`/freedoom2/MAP01` was not found")
		.children()
		.expect("`/freedoom2/MAP01` is not a directory")
		.enumerate()
	{
		assert_eq!(child.path_str(), EXPECTED_CHILDREN[index]);
	}
}

#[test]
fn glob() {
	let Some(req) = request() else {
		return;
	};

	let mut vfs = VirtualFs::default();

	if vfs.mount(req).total_err_count() > 0 {
		panic!("VFS glob unit test encountered mount errors");
	}

	{
		let glob = globset::Glob::new("/freedoom2/FCGRATE*").unwrap();
		let count = vfs.glob_par(glob).count();
		assert_eq!(
			count, 2,
			"expected 2 entries matching glob `/freedoom2/FCGRATE*`, found: {}",
			count
		);
	}

	{
		let glob = globset::Glob::new("/freedoom1/E*M[0123456789]").unwrap();
		let count = vfs.glob_par(glob).count();
		assert_eq!(count, 36, "expected 36 maps, found: {}", count);
	}

	{
		let glob = globset::Glob::new("/freedoom2/MAP*").unwrap();
		let count = vfs.glob_par(glob).count();
		assert_eq!(
			count, 352,
			"expected 352 maps and sub-entries, found: {}",
			count
		);
	}
}

// Details /////////////////////////////////////////////////////////////////////

#[must_use]
fn request() -> Option<MountRequest> {
	let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sample");

	let load_order = vec![
		(base.join("freedoom1.wad"), VPathBuf::from("/freedoom1")),
		(base.join("freedoom2.wad"), VPathBuf::from("/freedoom2")),
	];

	for (real_path, _) in &load_order {
		if !real_path.exists() {
			eprintln!(
				"VFS testing depends on the following files of sample data:\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom1.wad`\r\n\t\
				- `$CARGO_MANIFEST_DIR/sample/freedoom2.wad`\r\n\t\
				They can be acquired from https://freedoom.github.io/."
			);

			return None;
		}
	}

	Some(MountRequest {
		load_order,
		tracker: None,
		basedata: true,
	})
}
