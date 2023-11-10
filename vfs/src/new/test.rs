use super::*;

#[test]
fn vpath_smoke() {
	let root_vpb = VPathBuf::from("/");

	assert_eq!(root_vpb, VPathBuf::from("/"));
	assert_eq!(root_vpb, VPath::new("/"));
	assert!(root_vpb.components().next().is_none());

	let vpb = VPathBuf::from("/lorem/ipsum/dolor/sit.amet");

	let mut components = vpb.components();
	assert_eq!(components.next(), Some(VPath::new("lorem")));
	assert_eq!(components.next(), Some(VPath::new("ipsum")));
	assert_eq!(components.next(), Some(VPath::new("dolor")));
	assert_eq!(components.next(), Some(VPath::new("sit.amet")));
	assert_eq!(components.next(), None);

	let lmp = VPathBuf::from("/somewad/LuMp.A.b");
	assert_eq!(lmp.file_prefix().unwrap().as_str(), "LuMp");
	assert_eq!(lmp.file_stem().unwrap().as_str(), "LuMp.A");
	assert_eq!(lmp.extension().unwrap(), "b");
	assert_eq!(lmp.lump_name().unwrap().as_str(), "LUMP.A");

	let Some(vfs) = sample_vfs() else {
		return;
	};

	let lump = vfs.get(VPath::new("/freedoom2/FCGRATE2")).unwrap();

	let lmp_path = lump.path();
	assert_eq!(lmp_path, VPathBuf::from("/freedoom2/fcgrate2"));
	assert!(lmp_path.extension().is_none());
}

#[test]
fn mount_smoke() {
	let Some(_) = sample_vfs() else {
		return;
	};
}

#[test]
fn lookup_smoke() {
	let Some(vfs) = sample_vfs() else {
		return;
	};

	{
		let r = vfs.get(VPath::new("/")).unwrap();
		assert_eq!(r.into_folder().unwrap(), vfs.root());
	}

	{
		let r = vfs.get(VPath::new("//")).unwrap();
		assert_eq!(r.into_folder().unwrap(), vfs.root());
	}

	const SAMPLES: &[&str] = &[
		"freedoom2",
		"/freedoom2",
		"FREEDOOM2",
		"/FREEDOOM2",
		"/freedoom2/fcgrate2",
		"/freedoom2/FCGRATE2",
		"/FREEDOOM2/fcgrate2",
	];

	for sample in SAMPLES {
		let r = vfs.get(VPath::new(sample));
		assert!(r.is_ok(), "failed to look up `{sample}`");
	}
}

#[test]
fn read_smoke() {
	let Some(vfs) = sample_vfs() else {
		return;
	};

	assert!(vfs.get(VPath::new("/viletech.sf2")).is_ok());
	assert!(vfs.get(VPath::new("/viletech/viletech.png")).is_ok());

	let mut lump = vfs.get(VPath::new("/freedoom2/FCGRATE2")).unwrap();
	let bytes = lump.read().unwrap();

	assert_eq!(bytes.len(), 4096);

	assert_eq!(
		&bytes[..8],
		&[0x68, 0x6C, 0x6E, 0x6E, 0x6E, 0x6E, 0x6E, 0x6E]
	);

	assert_eq!(
		&bytes[4088..],
		&[0x6F, 0x6F, 0x6F, 0x05, 0x05, 0x6E, 0x68, 0x66]
	);
}

#[must_use]
fn sample_vfs() -> Option<VirtualFs> {
	let mut vfs = VirtualFs::default();

	let base = Path::new(env!("CARGO_MANIFEST_DIR"));

	{
		let sf2 = base.join("../assets/soundfonts/viletech.sf2");
		vfs.mount(&sf2, VPath::new("viletech.sf2")).unwrap();
	}

	{
		let basedata = base.join("../assets/viletech");
		vfs.mount(&basedata, VPath::new("viletech")).unwrap();
	}

	{
		let freedoom2 = base.join("../sample/freedoom2.wad");

		if !freedoom2.exists() {
			eprintln!(
				"No folder {} on the disk; skipping a test.",
				freedoom2.display()
			);

			return None;
		}

		vfs.mount(&freedoom2, VPath::new("freedoom2")).unwrap();
	}

	Some(vfs)
}
