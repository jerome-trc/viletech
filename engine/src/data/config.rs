use std::path::PathBuf;

use super::Catalog;

#[derive(Debug, Default)]
pub(super) struct Config {
	/// Mind that this stores real paths, and that its order matters.
	pub(super) basedata: Vec<PathBuf>,
}

/// Configuration methods are kept in a wrapper around a [`Catalog`] reference
/// to prevent bloat in the interface of the catalog itself.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConfigGet<'cat>(pub(super) &'cat Catalog);

impl ConfigGet<'_> {
	// ???
}

/// Configuration methods are kept in a wrapper around a [`Catalog`] reference
/// to prevent bloat in the interface of the catalog itself.
#[derive(Debug)]
#[repr(transparent)]
pub struct ConfigSet<'cat>(pub(super) &'cat mut Catalog);

impl ConfigSet<'_> {
	pub fn reserve_mount_point(self, mp: String) -> Self {
		self.0.vfs.config_set().reserve_mount_point(mp);
		self
	}
}
