//! # VileTech Engine
//!
//! VileTech is a pet project, meant as an experiment in creating a modern,
//! feature-oriented Doom source port in Rust which can successfully interpret
//! user-generated content for GZDoom and the Eternity Engine with zero end-user
//! overhead and minimal runtime overhead.

use once_cell::sync::OnceCell;

pub mod audio;
pub mod audio2;
pub mod console;
pub mod data;
pub mod devgui;
pub mod frontend;
pub mod gfx;
pub mod input;
pub extern crate level;
pub mod log;
pub extern crate mus;
pub mod player;
pub mod rng;
pub mod sim;
pub mod sparse;
pub mod terminal;
pub mod user;
pub extern crate lith;
pub extern crate util;
pub extern crate vfs;
pub extern crate wadload;

// Types ///////////////////////////////////////////////////////////////////////

/// See [`bevy::render::color::Color::Rgba`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbaF32 {
	pub red: f32,
	pub green: f32,
	pub blue: f32,
	pub alpha: f32,
}

/// Type alias for Bevy's two-point rectangle to disambiguate from VileTech's [4-point counterpart].
///
/// [4-point counterpart]: util::math::Rect4
pub type Rect2 = bevy::math::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BaseGame {
	Doom,
	Hexen,
	Heretic,
	Strife,
	ChexQuest,
}

// Constants ///////////////////////////////////////////////////////////////////

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_HASH: &str = env!("GIT_HASH");
pub const COMPILE_DATETIME: &str = env!("COMPILE_DATETIME");

// Symbols that don't belong in any other module ///////////////////////////////

/// Ideally setting this is the first operation your application makes.
/// Used by the [`log::TracingPlugin`] for formatting time in log messages.
pub static START_TIME: OnceCell<std::time::Instant> = OnceCell::new();

/// Prepares the rayon global thread pool. See [`rayon::ThreadPoolBuilder`].
/// If `num_threads` is `None` then rayon chooses it automatically.
/// This also ensures that these threads have clear names for debugging purposes.
pub fn thread_pool_init(num_threads: Option<usize>) {
	rayon::ThreadPoolBuilder::new()
		.thread_name(|index| format!("vile-global-{index}"))
		.num_threads(num_threads.unwrap_or(0))
		.build_global()
		.expect("failed to build Rayon's global thread pool")
}

/// Returns a message telling the user the engine's `CARGO_PKG_VERSION`.
#[must_use]
pub fn short_version_string() -> String {
	format!("VileTech Engine {}", env!("CARGO_PKG_VERSION"))
}

#[must_use]
pub fn version_info() -> [String; 3] {
	[
		short_version_string(),
		format!("Commit {GIT_HASH}"),
		format!("Compiled on {COMPILE_DATETIME}"),
	]
}

pub const BASEDATA_ID: &str = env!("BASEDATA_ID");
pub const BASEDATA_FILENAME: &str = env!("BASEDATA_FILENAME");

/// Panics if:
/// - In release mode, and the executable path can't be retrieved.
/// - In debug mode, and the working directory path can't be retrieved.
#[must_use]
pub fn basedata_path() -> std::path::PathBuf {
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
			PathBuf::from("data/viletech"),
		]
		.iter()
		.collect();
	}

	path
}

/// Returns an error if the engine's base data can't be found,
/// or has been tampered with somehow since application installation.
///
/// In the debug build, this looks for `$PWD/data/viletech`, and returns `Ok`
/// simply if that directory is found.
/// In the release build, this looks for `<exec dir>/viletech.zip`, and
/// ensures it's present and matches a checksum.
pub fn basedata_is_valid() -> Result<(), BaseDataError> {
	use sha3::Digest;
	use std::io::Read;
	use BaseDataError as Error;

	let path = basedata_path();

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
	"vile",
	"viletec",
	"vt",
	"vtec",
	"vtech",
	"viletech",
	"lith",
	"lithscript",
	"zs",
	"zscript",
];

#[derive(Debug)]
pub enum BaseDataError {
	Missing,
	ReadFailure(std::io::Error),
	ChecksumMismatch,
}

impl std::error::Error for BaseDataError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::ReadFailure(err) => Some(err),
			_ => None,
		}
	}
}

impl std::fmt::Display for BaseDataError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let path = basedata_path();
		let p = path.display();

		match self {
			Self::Missing => write!(f, "engine base data not found at `{p}`"),
			Self::ReadFailure(err) => err.fmt(f),
			Self::ChecksumMismatch => write!(f, "engine base data at `{p}` is corrupted"),
		}
	}
}

/// Allows upcasting from `dyn T` to [`std::any::Any`].
pub trait AsAny: std::any::Any {
	#[must_use]
	fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: std::any::Any> AsAny for T {
	fn as_any(&self) -> &dyn std::any::Any {
		// (RAT) As silly as this seems, it is the only halfway-elegant way of
		// doing this, and it has some surprisingly useful niche applications.
		self
	}
}
