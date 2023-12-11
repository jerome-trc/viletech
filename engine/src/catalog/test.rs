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

#[test]
fn version_from_string() {
	let mut input = [
		"DOOM".to_string(),
		"DOOM2".to_string(),
		"weighmedown_1.2.0".to_string(),
		"none an island V1.0".to_string(),
		"bitter_arcsV0.9.3".to_string(),
		"stuck-in-the-system_v3.1".to_string(),
		"555-5555 v5a".to_string(),
		"yesterdays_pain_6-19-2022".to_string(),
		"i-am_a dagger_1.3".to_string(),
		"BROKEN_MANTRA_R3.1c".to_string(),
		"There Is Still Time 0.3tf1".to_string(),
		"setmefree-4.7.1c".to_string(),
		"Outoftheframe_1_7_0b".to_string(),
		"a c i d r a i n_716".to_string(),
	];

	let expected = [
		("DOOM", ""),
		("DOOM2", ""),
		("weighmedown", "1.2.0"),
		("none an island", "V1.0"),
		("bitter_arcs", "V0.9.3"),
		("stuck-in-the-system", "v3.1"),
		("555-5555", "v5a"),
		("yesterdays_pain", "6-19-2022"),
		("i-am_a dagger", "1.3"),
		("BROKEN_MANTRA", "R3.1c"),
		("There Is Still Time", "0.3tf1"),
		("setmefree", "4.7.1c"),
		("Outoftheframe", "1_7_0b"),
		("a c i d r a i n", "716"),
	];

	for (i, string) in input.iter_mut().enumerate() {
		let res = super::version_from_string(string);

		if expected[i].1.is_empty() {
			assert!(
				res.is_none(),
				"[{i}] expected nothing; returned: {}",
				res.unwrap()
			);
		} else {
			assert!(
				string == expected[i].0,
				"[{i}] expected modified string: {} - actual output: {}",
				expected[i].0,
				string
			);

			let vers = res.unwrap();

			assert!(
				vers == expected[i].1,
				"[{i}] expected return value: {} - actual return value: {}",
				expected[i].1,
				vers
			);
		}
	}
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
