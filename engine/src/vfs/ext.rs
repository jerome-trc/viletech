//! VileTech-specific functionality.

use std::{
	fmt::Write,
	path::{Path, PathBuf},
	time::Instant,
};

use doom_front::zscript::filesystem::{File as ZsFile, FileSystem as ZsFileSystem};
use log::{error, info, warn};

use crate::{
	data::{MountKind, MountMetaIngest},
	utils::{path::*, string::*},
};

use super::{entry::PathHash, EntryKind, FileRef, VirtualFs, RGX_INVALIDMOUNTPATH};

/// A separate trait provides functions that are specific to VileTech, so that the
/// VFS itself can later be more easily made into a standalone library.
pub trait VirtualFsExt {
	/// On the debug build, attempt to mount `/env::current_dir()/data`.
	/// On the release build, attempt to mount `/utils::exe_dir()/viletech.zip`.
	fn mount_basedata(&mut self) -> Result<(), Box<dyn std::error::Error>>;
	#[must_use]
	fn mount_gamedata(&mut self, paths: &[PathBuf]) -> Vec<MountMetaIngest>;

	/// See [`FileRefExt::is_viletech_package`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn is_viletech_package(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`FileRefExt::is_udmf_map`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn is_udmf_map(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`FileRefExt::has_zscript`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn has_zscript(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`FileRefExt::has_edfroot`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn has_edfroot(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`FileRefExt::has_decorate`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn has_decorate(&self, path: impl AsRef<Path>) -> Option<bool>;

	#[must_use]
	fn gamedata_kind(&self, virtual_path: impl AsRef<Path>) -> MountKind;

	fn parse_gamedata_meta(
		&self,
		path: impl AsRef<Path>,
	) -> Result<MountMetaIngest, Box<dyn std::error::Error>>;

	#[must_use]
	fn window_icon_from_file(&self, path: impl AsRef<Path>) -> Option<winit::window::Icon>;

	#[must_use]
	fn ccmd_file(&self, path: PathBuf) -> String;
}

impl VirtualFsExt for VirtualFs {
	fn mount_basedata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		if let Err(err) = crate::basedata_is_valid() {
			return Err(Box::new(err));
		}

		if let Err(err) = self
			.mount(&[(crate::basedata_path(), "/viletech")])
			.pop()
			.unwrap()
		{
			Err(Box::new(err))
		} else {
			Ok(())
		}
	}

	fn mount_gamedata(&mut self, paths: &[PathBuf]) -> Vec<MountMetaIngest> {
		let call_time = Instant::now();
		let mut to_mount = Vec::<(&Path, PathBuf)>::with_capacity(paths.len());
		let mut vers_strings = Vec::<String>::with_capacity(paths.len());
		let mut ret = Vec::<MountMetaIngest>::with_capacity(paths.len());

		for real_path in paths {
			if real_path.is_symlink() {
				info!(
					"Skipping game data object for mount: {}\r\n\t\
					Reason: mounting symbolic links is forbidden.",
					real_path.display()
				);
				continue;
			}

			let mount_point =
				if real_path.is_dir() || real_path.is_supported_archive().unwrap_or_default() {
					let osfstem = real_path.file_stem();

					if osfstem.is_none() {
						warn!(
							"Skipping gamedata entry (invalid file stem): {}",
							real_path.display()
						);
						continue;
					}

					let fstem = osfstem.unwrap().to_str();

					if fstem.is_none() {
						warn!(
							"Skipping gamedata entry (invalid UTF-8 in name): {}",
							real_path.display()
						);
						continue;
					}

					fstem.unwrap()
				} else if !real_path.is_binary().unwrap_or(true) {
					let fname = real_path.file_name();
					let fname = fname.unwrap_or_default().to_str();

					if fname.is_none() {
						warn!(
							"Skipping gamedata entry (invalid UTF-8 in name): {}",
							real_path.display()
						);
						continue;
					}

					fname.unwrap()
				} else {
					warn!(
						"Skipping unsupported/non-existent gamedata entry: {}",
						real_path.display()
					);
					continue;
				}
				.replace(' ', "_");

			let mut mount_point = RGX_INVALIDMOUNTPATH
				.replace_all(&mount_point, "")
				.to_string();

			let vers = version_from_string(&mut mount_point);
			vers_strings.push(vers.unwrap_or_default());
			to_mount.push((real_path, PathBuf::from(&mount_point)));
		}

		let results = self.mount(&to_mount[..]);
		debug_assert!(to_mount.len() == vers_strings.len());

		for (i, res) in results.iter().enumerate() {
			if res.is_err() {
				// No error messaging here:
				// should already have been reported by `mount()`
				continue;
			}

			// If we mount `foo` and then can't find `foo`, things are REALLY bad
			let is_vile_pkg = self
				.is_viletech_package(&to_mount[i].1)
				.expect("Failed to look up a newly-mounted item.");

			let meta = if is_vile_pkg {
				let metapath: PathBuf = [PathBuf::from(&to_mount[i].1), PathBuf::from("meta.toml")]
					.iter()
					.collect();

				match self.parse_gamedata_meta(&metapath) {
					Ok(mut m) => {
						m.virt_path = to_mount[i].1.clone();
						m
					}
					Err(err) => {
						error!(
							"Failed to parse gamedata meta file for package: {}\r\n\t\
							Error: {}",
							to_mount[i].0.display(),
							err
						);
						continue;
					}
				}
			} else {
				let id = to_mount[i].1.to_string_lossy().to_string();
				let version = vers_strings.remove(0);

				MountMetaIngest {
					id,
					version,
					virt_path: to_mount[i].1.clone(),
					..Default::default()
				}
			};

			ret.push(meta);
		}

		info!(
			"Mounted {} game data object(s) in {} ms.",
			results.len(),
			call_time.elapsed().as_millis()
		);

		ret
	}

	fn is_viletech_package(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|file| file.is_viletech_package())
	}

	fn is_udmf_map(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|file| file.is_udmf_map())
	}

	fn has_zscript(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|file| file.has_zscript())
	}

	fn has_edfroot(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|file| file.has_edfroot())
	}

	fn has_decorate(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|file| file.has_decorate())
	}

	fn gamedata_kind(&self, virtual_path: impl AsRef<Path>) -> MountKind {
		fn check_path(path: &Path) -> std::option::Option<MountKind> {
			if path.has_gzdoom_extension() {
				return Some(MountKind::ZDoom);
			}

			if path.has_eternity_extension() {
				return Some(MountKind::Eternity);
			}

			if path.has_wad_extension() {
				// TODO: Hash known IWADs, compare against those
				return Some(MountKind::Wad { internal: false });
			}

			None
		}

		let entry = &self
			.entries
			.get(&PathHash::new(virtual_path))
			.expect("`VirtualFsExt::gamedata_kind` received a non-existent virtual path.");

		match &entry.kind {
			EntryKind::Directory(..) => {
				let real_path = self
					.virtual_to_real(&entry.path)
					.expect("`VirtualFsExt::gamedata_kind` failed to resolve a real path.");

				for child in self.children_of(entry) {
					if !child.file_name().eq_ignore_ascii_case("meta.toml") {
						continue;
					}

					if child.is_dir() {
						continue;
					}

					if child.is_binary() {
						warn!(
							"Invalid meta.toml file under `{}`: expected string entry, found binary entry.",
							entry.path_str(),
						);
						continue;
					}

					let meta: MountMetaIngest = match toml::from_str(child.read_str()) {
						Ok(m) => m,
						Err(err) => {
							warn!(
								"Failed to read game data meta file: {}\r\n\t\
								Error: {}",
								entry.path_str(),
								err
							);
							continue;
						}
					};

					match meta.manifest {
						Some(pb) => return MountKind::VileTech { manifest: pb },
						None => {
							return MountKind::VileTech {
								manifest: PathBuf::default(),
							}
						}
					}
				}

				if let Some(kind) = check_path(&real_path) {
					return kind;
				}
			}
			_ => {
				return MountKind::File;
			}
		};

		// Either the user has done something unexpected
		// or we just need more heuristics I haven't come up with yet
		unreachable!();
	}

	fn parse_gamedata_meta(
		&self,
		path: impl AsRef<Path>,
	) -> Result<MountMetaIngest, Box<dyn std::error::Error>> {
		let text = self.read_str(path.as_ref())?;
		let ret: MountMetaIngest = toml::from_str(text)?;
		Ok(ret)
	}

	fn window_icon_from_file(&self, path: impl AsRef<Path>) -> Option<winit::window::Icon> {
		let bytes = match self.read(path) {
			Ok(b) => b,
			Err(err) => {
				error!("Failed to read engine icon image bytes: {err}");
				return None;
			}
		};

		let icon = match image::load_from_memory(bytes) {
			Ok(i) => i,
			Err(err) => {
				error!("Failed to load engine icon: {err}");
				return None;
			}
		}
		.into_rgba8();

		let (width, height) = icon.dimensions();
		let rgba = icon.into_raw();

		match winit::window::Icon::from_rgba(rgba, width, height) {
			Ok(r) => Some(r),
			Err(err) => {
				error!("Failed to create winit icon from image data: {err}");
				None
			}
		}
	}

	fn ccmd_file(&self, path: PathBuf) -> String {
		let file = match self.lookup(&path) {
			Some(e) => e,
			None => {
				return "Nothing exists at that path.".to_string();
			}
		};

		if !file.is_dir() {
			return format!(
				"{}\r\n\tSize: {}B",
				file.file_name(),
				file.try_read().unwrap().len()
			);
		}

		let child_count = file.child_count();
		let mut ret = String::with_capacity(child_count * 32);

		for child in file.child_entries() {
			match write!(ret, "\r\n\t{}", child.file_name()) {
				Ok(()) => {}
				Err(err) => {
					warn!(
						"Failed to write an output line for ccmd.: `file`\r\n\t\
						Error: {}",
						err
					);
				}
			}

			if child.is_dir() {
				ret.push('/');
			}
		}

		format!(
			"Files under \"{}\" ({}): {}",
			path.display(),
			child_count,
			ret
		)
	}
}

/// A separate trait provides functions that are specific to VileTech, so that the
/// VFS itself can later be more easily made into a standalone library.
pub trait FileRefExt {
	/// Check if a directory node has a `meta.toml` leaf (case-insensitive) in it.
	/// Unconditionally returns false if the file's entry is, itself, a leaf node.
	#[must_use]
	fn is_viletech_package(&self) -> bool;
	/// Check if this is a directory with a leaf node named `TEXTMAP`.
	/// Unconditionally returns false if the file's entry is, itself, a leaf node.
	#[must_use]
	fn is_udmf_map(&self) -> bool;
	/// Check if a directory node has a `decorate` file (case-insensitive) in it.
	/// Unconditionally returns false if the file's entry is, itself, a leaf node.
	#[must_use]
	fn has_decorate(&self) -> bool;
	/// Check if a directory node has a `zscript` file (case-insensitive) in it.
	/// Unconditionally returns false if the file's entry is, itself, a leaf node.
	#[must_use]
	fn has_zscript(&self) -> bool;
	/// Check if a directory node has an `edfroot` file (case-insensitive) in it.
	/// Unconditionally returns false if the file's entry is, itself, a leaf node.
	#[must_use]
	fn has_edfroot(&self) -> bool;
}

impl FileRefExt for FileRef<'_> {
	fn is_viletech_package(&self) -> bool {
		self.is_dir()
			&& self.child_entries().any(|e| {
				e.path
					.file_stem()
					.unwrap_or_default()
					.eq_ignore_ascii_case("meta.toml")
			})
	}

	fn is_udmf_map(&self) -> bool {
		self.child_entries().any(|e| e.file_name() == "TEXTMAP")
	}

	fn has_decorate(&self) -> bool {
		self.is_dir()
			&& self.child_entries().any(|e| {
				let stem = e
					.path
					.file_stem()
					.unwrap_or_default()
					.to_str()
					.unwrap_or_default();
				stem.split('.')
					.next()
					.unwrap_or_default()
					.eq_ignore_ascii_case("edfroot")
			})
	}

	fn has_zscript(&self) -> bool {
		self.is_dir()
			&& self.child_entries().any(|e| {
				let stem = e
					.path
					.file_stem()
					.unwrap_or_default()
					.to_str()
					.unwrap_or_default();
				stem.split('.')
					.next()
					.unwrap_or_default()
					.eq_ignore_ascii_case("zscript")
			})
	}

	fn has_edfroot(&self) -> bool {
		self.is_dir()
			&& self.child_entries().any(|e| {
				let stem = e
					.path
					.file_stem()
					.unwrap_or_default()
					.to_str()
					.unwrap_or_default();
				stem.split('.')
					.next()
					.unwrap_or_default()
					.eq_ignore_ascii_case("edfroot")
			})
	}
}

impl ZsFileSystem for FileRef<'_> {
	fn get_file(&mut self, filename: &str) -> Option<ZsFile> {
		let target = match self.child_entries().find(|e| e.path_str() == filename) {
			Some(h) => h,
			None => {
				let full_path = self.path.join(filename);
				warn!("Failed to find ZScript file: {}", full_path.display());
				return None;
			}
		};

		if target.is_dir() {
			let full_path = self.path.join(filename);
			warn!(
				"Expected ZScript file, found directory: {}",
				full_path.display()
			);
			return None;
		}

		Some(ZsFile::new(filename.to_string(), target.clone().unwrap()))
	}

	fn get_files_no_ext(&mut self, filename: &str) -> Vec<ZsFile> {
		let mut ret = Vec::<ZsFile>::default();

		for child in self.child_entries() {
			let mut noext = child.file_name().splitn(2, '.');

			let bytes = if child.is_leaf() {
				child.try_read().unwrap()
			} else {
				continue;
			};

			let stem = match noext.next() {
				Some(s) => s,
				None => {
					continue;
				}
			};

			if stem.eq_ignore_ascii_case(filename) {
				ret.push(ZsFile::new(filename.to_string(), bytes.to_vec()));
			}
		}

		ret
	}
}
