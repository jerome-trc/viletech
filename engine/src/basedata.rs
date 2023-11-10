//! Functions related to the "engine basedata":
//! assets required for VileTech applications to function.

pub const BASEDATA_ID: &str = env!("BASEDATA_ID");
pub const BASEDATA_FILENAME: &str = env!("BASEDATA_FILENAME");

/// Panics if:
/// - In release mode, and the executable path can't be retrieved.
/// - In debug mode, and the working directory path can't be retrieved.
#[must_use]
pub fn path() -> std::path::PathBuf {
	use std::path::PathBuf;

	let path: PathBuf;

	#[cfg(not(debug_assertions))]
	{
		path = [util::path::exe_dir(), PathBuf::from(BASEDATA_FILENAME)]
			.iter()
			.collect();
	}
	#[cfg(debug_assertions)]
	{
		path = [
			std::env::current_dir().expect("failed to get working directory"),
			PathBuf::from("assets/viletech"),
		]
		.iter()
		.collect();
	}

	path
}

/// Returns an error if the engine's base data can't be found,
/// or has been tampered with somehow since application installation.
///
/// In the debug build, this looks for `$PWD/assets/viletech`, and returns `Ok`
/// simply if that directory is found.
/// In the release build, this looks for `<exec dir>/viletech.zip`, and
/// ensures it's present and matches a checksum.
pub fn is_valid() -> Result<(), Error> {
	use sha3::Digest;
	use std::io::Read;

	let path = path();

	if !path.exists() {
		return Err(Error::Missing);
	}

	if cfg!(debug_assertions) {
		return Ok(());
	}

	let mut file = std::fs::File::open(path).map_err(Error::ReadFailure)?;
	let file_len = file.metadata().map_err(Error::ReadFailure)?.len() as usize;
	let mut zip_bytes = Vec::with_capacity(file_len);
	file.read_to_end(&mut zip_bytes)
		.map_err(Error::ReadFailure)?;

	let mut hasher = sha3::Sha3_256::new();
	hasher.update(&zip_bytes[..]);
	let checksum = hasher.finalize();
	let mut string = String::with_capacity(checksum.len());

	for byte in checksum {
		string.push_str(&byte.to_string());
	}

	#[cfg_attr(debug_assertions, allow(clippy::comparison_to_empty))]
	if string == env!("BASEDATA_CHECKSUM") {
		Ok(())
	} else {
		Err(Error::ChecksumMismatch)
	}
}

pub const RESERVED_MOUNT_POINTS: &[&str] = &[
	"vile", "viletec", "vt", "vtec", "vtech", "viletech", "lith", "lithica", "zs", "zscript",
];

/// See [`basedata_is_valid`].
#[derive(Debug)]
pub enum Error {
	Missing,
	ReadFailure(std::io::Error),
	ChecksumMismatch,
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::ReadFailure(err) => Some(err),
			_ => None,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let path = path();
		let p = path.display();

		match self {
			Self::Missing => write!(f, "engine base data not found at `{p}`"),
			Self::ReadFailure(err) => err.fmt(f),
			Self::ChecksumMismatch => write!(f, "engine base data at `{p}` is corrupted"),
		}
	}
}
