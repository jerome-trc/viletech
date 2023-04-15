//! Not-so-general functionality is provided via extension traits.
//!
//! Keeps the more crucial namespaces cleaner and makes it easier to reuse the
//! data management code, in case it proves robust enough for other projects.

use bevy::prelude::{error, warn};

use crate::BaseDataError;

use super::{Catalog, LoadOutcome, LoadRequest};

pub trait CatalogExt {
	/// On the debug build, attempt to load `/env::current_dir()/data/viletech`.
	/// On the release build, attempt to load `/utils::exe_dir()/viletech.vpk3`.
	fn mount_basedata(&mut self) -> Result<(), BaseDataError>;

	// TODO: Re-enable this helper when Bevy supports it.
	// See: https://github.com/bevyengine/bevy/issues/1031

	#[cfg(any())]
	fn window_icon_from_file(
		&self,
		path: impl AsRef<VPath>,
	) -> Result<winit::window::Icon, Box<dyn std::error::Error>>;
}

impl CatalogExt for Catalog {
	fn mount_basedata(&mut self) -> Result<(), BaseDataError> {
		crate::basedata_is_valid()?;

		let req = LoadRequest {
			load_order: vec![(crate::basedata_path(), "/viletech")],
			tracker: None,
			dev_mode: false,
		};

		// TODO: Base data may get split into more packages, so refine
		// this later, when the situation is clearer.
		match self.load(req) {
			LoadOutcome::MountFail { mut errors } => {
				for err in errors.pop().unwrap() {
					error!("{err}");
				}

				Err(BaseDataError::Load)
			}
			LoadOutcome::PrepFail { mut errors } => {
				for err in errors.pop().unwrap() {
					error!("{err}");
				}

				Err(BaseDataError::Load)
			}
			LoadOutcome::Ok {
				mut mount,
				mut prep,
			} => {
				for err in mount.pop().unwrap() {
					warn!("{err}");
				}

				for err in prep.pop().unwrap() {
					warn!("{err}");
				}

				Ok(())
			}
			other => unreachable!("Impossible base data load result: {other:#?}"),
		}
	}

	#[cfg(any())]
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
