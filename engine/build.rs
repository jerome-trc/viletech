use std::{
	fs::File,
	io::{Read, Write},
	path::{Path, PathBuf},
	process::Command,
};

use sha3::{Digest, Sha3_256};

const BASEDATA_ID: &str = "viletech";
const BASEDATA_FILENAME: &str = "viletech.vpk3";

/// - Injects the current Git hash and date and time of compilation
/// into the environment before building.
/// - Generates `viletech.vpk3` (a zip archive), known as the "base data".
fn main() -> miette::Result<(), Box<dyn std::error::Error>> {
	let hash = match Command::new("git").args(["rev-parse", "HEAD"]).output() {
		Ok(h) => h,
		Err(err) => {
			eprintln!("Failed to execute `git rev-parse HEAD`: {err}");
			return Err(Box::new(err));
		}
	};

	let hash_str = match String::from_utf8(hash.stdout) {
		Ok(s) => s,
		Err(err) => {
			eprintln!("Failed to convert output of `git rev-parse HEAD` to UTF-8: {err}",);
			return Err(Box::new(err));
		}
	};

	let fmt =
		time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();
	let compile_timestamp = time::OffsetDateTime::now_utc().format(&fmt).unwrap();

	println!("cargo:rustc-env=GIT_HASH={hash_str}");
	println!("cargo:rustc-env=COMPILE_DATETIME={compile_timestamp} UTC");

	println!("cargo:rustc-env=BASEDATA_ID={BASEDATA_ID}");
	println!("cargo:rustc-env=BASEDATA_FILENAME={BASEDATA_FILENAME}");
	// Empty in debug mode. Gets filled by `build_basedata`.
	println!("cargo:rustc-env=BASEDATA_CHECKSUM=");

	if std::env::var("PROFILE").unwrap() == "release" {
		build_basedata()?;
	}

	Ok(())
}

/// Compile the contents of `/data/viletech` into `/target/viletech.vpk3`.
/// Hash the bytes of that file, and store the stringifiedhash in an
/// environment variable that gets compiled into the engine.
fn build_basedata() -> Result<(), Box<dyn std::error::Error>> {
	let data_path: PathBuf = [env!("CARGO_WORKSPACE_DIR"), "data", "viletech"]
		.iter()
		.collect::<PathBuf>();

	if !data_path.exists() {
		panic!("Base data directory not found.");
	}

	let pkg_path = [
		env!("CARGO_WORKSPACE_DIR"),
		"target",
		"release",
		BASEDATA_FILENAME,
	]
	.iter()
	.collect::<PathBuf>();

	let options = zip::write::FileOptions::default().compression_level(Some(9));

	let file = File::create(&pkg_path)?;
	let mut zip = zip::ZipWriter::new(file);

	let walker = walkdir::WalkDir::new::<&Path>(&data_path)
		.follow_links(false)
		.max_depth(16)
		.same_file_system(true)
		.sort_by_file_name()
		.into_iter();

	let mut buffer = Vec::with_capacity(1024 * 1024 * 16);

	for dir_entry in walker {
		let dir_entry = dir_entry?;
		let metadata = dir_entry.metadata()?;

		let path = dir_entry.path();

		let path_rel = match path.strip_prefix(&data_path)?.to_str() {
			Some(s) => s.to_string(),
			None => {
				let p = path.display();
				panic!("Base data file has path with invalid UTF-8: {p}");
			}
		};

		if metadata.is_dir() {
			zip.add_directory(path_rel, options)?;
			continue;
		}

		let file_len = metadata.len() as usize;

		zip.start_file(path_rel, options)?;
		let mut f = File::open(path)?;
		f.read_to_end(&mut buffer)?;
		let written = zip.write(&buffer[..])?;

		assert!(
			written == file_len,
			"Expected to write {file_len} bytes, wrote {written}.",
		);

		buffer.clear();
	}

	zip.finish()?;

	let mut file = File::open(&pkg_path)?;
	let file_len = file.metadata()?.len() as usize;
	let mut zip_bytes = Vec::with_capacity(file_len);
	file.read_to_end(&mut zip_bytes)?;

	let mut hasher = Sha3_256::new();
	hasher.update(&zip_bytes[..]);
	let checksum = hasher.finalize();
	let mut string = String::with_capacity(checksum.len());

	for byte in checksum {
		string.push_str(&byte.to_string());
	}

	println!("cargo:rustc-env=BASEDATA_CHECKSUM={string}");

	Ok(())
}
