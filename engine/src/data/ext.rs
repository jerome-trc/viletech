//! Not-so-general functionality is provided via extension traits.
//!
//! Keeps the more crucial namespaces cleaner and makes it easier to reuse the
//! data management code, in case it proves robust enough for other projects.

use crate::VPath;

use super::{Catalog, VfsError};

pub trait CatalogExt {
	/// On the debug build, attempt to mount `/env::current_dir()/data`.
	/// On the release build, attempt to mount `/utils::exe_dir()/viletech.zip`.
	fn mount_basedata(&mut self) -> Result<(), Box<dyn std::error::Error>>;

	fn window_icon_from_file(
		&self,
		path: impl AsRef<VPath>,
	) -> Result<winit::window::Icon, Box<dyn std::error::Error>>;
}

impl CatalogExt for Catalog {
	fn mount_basedata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		if let Err(err) = crate::basedata_is_valid() {
			return Err(Box::new(err));
		}

		if let Err(err) = self
			.load_simple(&[(crate::basedata_path(), "/viletech")])
			.pop()
			.unwrap()
		{
			Err(Box::new(err))
		} else {
			Ok(())
		}
	}

	fn window_icon_from_file(
		&self,
		path: impl AsRef<VPath>,
	) -> Result<winit::window::Icon, Box<dyn std::error::Error>> {
		let path = path.as_ref();

		let file = self
			.get_file(path)
			.ok_or_else(|| Box::new(VfsError::NotFound(path.to_path_buf())))?;

		let bytes = file.try_read_bytes()?;
		let icon = image::load_from_memory(bytes)?.into_rgba8();
		let (width, height) = icon.dimensions();

		winit::window::Icon::from_rgba(icon.into_raw(), width, height).map_err(|err| {
			let b: Box<dyn std::error::Error> = Box::new(err);
			b
		})
	}
}
