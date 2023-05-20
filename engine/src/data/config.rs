use std::{collections::HashSet, path::PathBuf};

use super::Catalog;

#[derive(Debug)]
pub(super) struct Config {
	/// Mind that this stores real paths.
	pub(super) basedata: HashSet<PathBuf>,
	///
	pub(super) reserved_mount_points: Vec<String>,
	pub(super) bin_size_limit: usize,
	pub(super) text_size_limit: usize,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			basedata: HashSet::default(),
			reserved_mount_points: vec![],
			bin_size_limit: super::limits::DEFAULT_BIN_FILE_SIZE,
			text_size_limit: super::limits::DEFAULT_TEXT_FILE_SIZE,
		}
	}
}

/// Configuration methods are kept in a wrapper around a [`Catalog`] reference
/// to prevent bloat in the interface of the catalog itself.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConfigGet<'cat>(pub(super) &'cat Catalog);

impl ConfigGet<'_> {
	/// The limit on the size of a virtual binary file. Irrelevant to datum management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The returned value is in bytes, and defaults to [`limits::DEFAULT_BIN_FILE_SIZE`].
	#[must_use]
	pub fn bin_size_limit(&self) -> usize {
		self.0.config.bin_size_limit
	}

	/// The limit on the size of a virtual text file. Irrelevant to datum management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The returned value is in bytes, and defaults to [`limits::DEFAULT_TEXT_FILE_SIZE`].
	#[must_use]
	pub fn text_size_limit(&self) -> usize {
		self.0.config.text_size_limit
	}
}

/// Configuration methods are kept in a wrapper around a [`Catalog`] reference
/// to prevent bloat in the interface of the catalog itself.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConfigSet<'cat>(pub(super) &'cat mut Catalog);

impl ConfigSet<'_> {
	/// The limit on the size of a virtual binary file. Irrelevant to datum management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The parameter is in bytes, and gets clamped between 0 and
	/// [`limits::MAX_BIN_FILE_SIZE`]. The default is [`limits::DEFAULT_BIN_FILE_SIZE`].
	pub fn bin_size_limit(self, limit: usize) -> Self {
		self.0.config.bin_size_limit = limit.clamp(0, limits::MAX_BIN_FILE_SIZE);
		self
	}

	/// The limit on the size of a virtual text file. Irrelevant to datum management.
	/// A mount can be any size the physical filesystem can handle, but any children
	/// over this size are rejected with a logged warning.
	///
	/// The parameter is in bytes, and gets clamped between 0 and
	/// [`limits::MAX_TEXT_FILE_SIZE`]. The default is [`limits::DEFAULT_TEXT_FILE_SIZE`].
	pub fn text_size_limit(self, limit: usize) -> Self {
		self.0.config.text_size_limit = limit.clamp(0, limits::MAX_TEXT_FILE_SIZE);
		self
	}

	pub fn reserve_mount_point(self, mp: String) -> Self {
		self.0.config.reserved_mount_points.push(mp);
		self
	}
}

pub mod limits {
	/// 1024 B * 1024 kB * 512 MB = 536870912 bytes.
	pub const DEFAULT_BIN_FILE_SIZE: usize = 1024 * 1024 * 512;
	/// 1024 B * 1024 kB * 64 MB = 67108864 bytes.
	pub const DEFAULT_TEXT_FILE_SIZE: usize = 1024 * 1024 * 64;
	/// 1024 B * 1024 kB * 1024 MB * 2 GB = 2147483648 bytes.
	pub const MAX_BIN_FILE_SIZE: usize = 1024 * 1024 * 1024 * 2;
	/// 1024 B * 1024 kB * 128 MB = 134217728 bytes.
	pub const MAX_TEXT_FILE_SIZE: usize = 1024 * 1024 * 128;

	// (RAT) If you guessed that the default text file size limit could
	// be much lower if not for the UDMF TEXTMAP format, then you're correct.
	// Ar Luminae's MAP01 TEXTMAP is 43.69 MB.
}
