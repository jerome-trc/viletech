use crate::{data::PackageType, utils::PathEx};
use log::{error, info, warn};
use physfs_rs::{PhysFs, PhysFsError};
use std::{
	error::Error,
	io::Read,
	path::{Path, PathBuf},
};

pub trait ImpureVfs {
	fn mount_ex(&mut self, path: &str, mount_point: &str) -> Result<(), PhysFsError>;
	fn read_string(&self, path: &Path) -> Result<String, Box<dyn Error>>;
	fn package_type(&self, path: &Path) -> PackageType;
	fn window_icon_from_file(&self, path: &Path) -> Option<winit::window::Icon>;
}

impl ImpureVfs for PhysFs {
	fn mount_ex(&mut self, path: &str, mount_point: &str) -> Result<(), PhysFsError> {
		if let Err(err) = self.mount(path, mount_point, true) {
			Err(err)
		} else {
			info!("VFS mounted '{}' as '{}'.", path, mount_point);
			Ok(())
		}
	}

	fn read_string(&self, path: &Path) -> Result<String, Box<dyn Error>> {
		let mut file = match self.open_read(path.to_string_lossy()) {
			Ok(f) => f,
			Err(err) => {
				return Err(Box::new(err));
			}
		};

		let file_len: usize = match file.file_length().try_into() {
			Ok(size) => size,
			Err(err) => {
				error!("File is too long to read ({}): {}.", err, path.display());
				return Err(Box::new(err));
			}
		};

		let mut ret = String::with_capacity(file_len);

		match file.read_to_string(&mut ret) {
			Ok(bytes) => {
				if bytes < file_len {
					warn!("Failed to read all bytes of file: {}", path.display());
				}

				Ok(ret)
			}
			Err(err) => Err(Box::new(err)),
		}
	}

	fn package_type(&self, path: &Path) -> PackageType {
		if path.extension_is(Path::new("pk3"))
			|| path.extension_is(Path::new("pk7"))
			|| path.extension_is(Path::new("ipk3"))
			|| path.extension_is(Path::new("ipk7"))
		{
			return PackageType::GzDoom;
		}

		if path.extension_is(Path::new("wad"))
			|| path.extension_is(Path::new("iwad"))
			|| path.extension_is(Path::new("pwad"))
		{
			return PackageType::Wad;
		}

		let mut mtdp = PathBuf::from(path);
		mtdp.push("meta.lua");
		if mtdp.exists() {
			return PackageType::Impure;
		}

		if path.extension_is(Path::new("pke")) {
			return PackageType::Eternity;
		}

		// TODO: According to the Eternity Engine wiki,
		// Eternity packages may be extended with pk3;
		// this will require further disambiguation

		PackageType::None
	}

	fn window_icon_from_file(&self, path: &Path) -> Option<winit::window::Icon> {
		let mut file = match self.open_read(path.to_string_lossy()) {
			Ok(f) => f,
			Err(err) => {
				error!("Failed to retrieve engine icon image: {}", err);
				return None;
			}
		};

		let bytes = match file.read_to_vec() {
			Ok(b) => b,
			Err(err) => {
				error!("Failed to read engine icon image bytes: {}", err);
				return None;
			}
		};

		let icon = match image::load_from_memory(&bytes[..]) {
			Ok(i) => i,
			Err(err) => {
				error!("Failed to load engine icon: {}", err);
				return None;
			}
		}
		.into_rgba8();

		let (width, height) = icon.dimensions();
		let rgba = icon.into_raw();

		match winit::window::Icon::from_rgba(rgba, width, height) {
			Ok(r) => Some(r),
			Err(err) => {
				error!("Failed to create winit icon from image data: {}", err);
				None
			}
		}
	}
}
