//! Not-so-general functionality is provided via extension traits.
//!
//! Keeps the more crucial namespaces cleaner and makes it easier to reuse the
//! data management code, in case it proves robust enough for other projects.

use log::error;

use crate::{BaseDataError, VPath};

use super::{Catalog, LoadRequest, VfsError};

pub trait CatalogExt {
	/// On the debug build, attempt to load `/env::current_dir()/data/viletech`.
	/// On the release build, attempt to load `/utils::exe_dir()/viletech.vpk3`.
	fn mount_basedata(&mut self) -> Result<(), BaseDataError>;

	fn window_icon_from_file(
		&self,
		path: impl AsRef<VPath>,
	) -> Result<winit::window::Icon, Box<dyn std::error::Error>>;
}

impl CatalogExt for Catalog {
	fn mount_basedata(&mut self) -> Result<(), BaseDataError> {
		crate::basedata_is_valid()?;

		let req = LoadRequest {
			paths: vec![(crate::basedata_path(), "/viletech")],
			tracker: None,
		};

		if let Err(errs) = self.load(req).pop().unwrap() {
			errs.iter().for_each(|err| error!("{err}"));
			Err(BaseDataError::Load(errs))
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
