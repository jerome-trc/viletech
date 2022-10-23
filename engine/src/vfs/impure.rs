//! Impure-specific functionality.

/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use lazy_static::lazy_static;
use log::{error, info, warn};
use regex::Regex;

use super::{Error, Handle, VirtualFs};

use crate::{
	data::{game::GameDataKind, GameDataMeta},
	utils::{path::*, string::*},
	vfs::{EntryKind, RGX_INVALIDMOUNTPATH},
	zscript::parser::fs::{File as ZsFile, FileSystem as ZsFileSystem},
};

/// A separate trait provides functions that are specific to Impure, so that the
/// VFS itself can later be more easily made into a standalone library.
pub trait ImpureVfs {
	/// On the debug build, attempt to mount `/env::current_dir()/data`.
	/// On the release build, attempt to mount `/utils::exe_dir()/impure.zip`.
	fn mount_enginedata(&mut self) -> Result<(), Error>;
	#[must_use]
	fn mount_gamedata(&mut self, paths: &[PathBuf]) -> Vec<GameDataMeta>;

	/// See [`ImpureVfsHandle::is_impure_package`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn is_impure_package(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`ImpureVfsHandle::is_udmf_map`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn is_udmf_map(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`ImpureVfsHandle::has_zscript`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn has_zscript(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`ImpureVfsHandle::has_edfroot`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn has_edfroot(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`ImpureVfsHandle::has_decorate`].
	/// Returns `None` if and only if nothing exists at the given path.
	#[must_use]
	fn has_decorate(&self, path: impl AsRef<Path>) -> Option<bool>;

	#[must_use]
	fn gamedata_kind(&self, uuid: &str) -> GameDataKind;

	fn parse_gamedata_meta(
		&self,
		path: impl AsRef<Path>,
	) -> Result<GameDataMeta, Box<dyn std::error::Error>>;

	#[must_use]
	fn window_icon_from_file(&self, path: impl AsRef<Path>) -> Option<winit::window::Icon>;

	#[must_use]
	fn ccmd_file(&self, path: PathBuf) -> String;
}

impl ImpureVfs for VirtualFs {
	fn mount_enginedata(&mut self) -> Result<(), Error> {
		let path: PathBuf;

		#[cfg(not(debug_assertions))]
		{
			path = [exe_dir(), PathBuf::from("impure.zip")].iter().collect();
		}
		#[cfg(debug_assertions)]
		{
			path = [
				std::env::current_dir().map_err(Error::IoError)?,
				PathBuf::from("data"),
			]
			.iter()
			.collect();
		}

		self.mount(&[(path, "/impure")]).pop().unwrap()
	}

	fn mount_gamedata(&mut self, paths: &[PathBuf]) -> Vec<GameDataMeta> {
		let call_time = Instant::now();
		let mut to_mount = Vec::<(&Path, PathBuf)>::with_capacity(paths.len());
		let mut vers_strings = Vec::<String>::with_capacity(paths.len());
		let mut ret = Vec::<GameDataMeta>::with_capacity(paths.len());

		for real_path in paths {
			if real_path.is_symlink() {
				info!(
					"Skipping game data object for mount: {}
					Reason: mounting symbolic links is forbidden",
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
							"Skipping gamedata entry (invalid Unicode in name): {}",
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
							"Skipping gamedata entry (invalid Unicode in name): {}",
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
		debug_assert!(results.len() == to_mount.len() && to_mount.len() == vers_strings.len());

		for (i, res) in results.iter().enumerate() {
			if res.is_err() {
				// No error messaging here:
				// should already have been reported by `mount()`
				continue;
			}

			// If we mount `foo` and then can't find `foo`, things are REALLY bad
			let is_impure_package = self
				.is_impure_package(&to_mount[i].1)
				.expect("Failed to look up a newly-mounted item.");

			let meta = if is_impure_package {
				let metapath: PathBuf = [PathBuf::from(&to_mount[i].1), PathBuf::from("meta.toml")]
					.iter()
					.collect();

				match self.parse_gamedata_meta(&metapath) {
					Ok(m) => m,
					Err(err) => {
						error!(
							"Failed to parse gamedata meta file for package: {}
							Error: {}",
							to_mount[i].0.display(),
							err
						);
						continue;
					}
				}
			} else {
				let uuid = to_mount[i].1.to_string_lossy().to_string();
				let vers = vers_strings.remove(0);
				let kind = self.gamedata_kind(&uuid);
				GameDataMeta::new(uuid, vers, kind)
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

	fn is_impure_package(&self, path: impl AsRef<Path>) -> Option<bool> {
		lazy_static! {
			static ref RGX_METATOML: Regex = Regex::new(r"^(?i)meta\.toml")
				.expect("Failed to evaluate `ImpureVfs::is_impure_package::RGX_METATOML`.");
		};

		self.lookup(path)
			.map(|handle| handle.is_dir() && handle.contains_regex(&RGX_METATOML))
	}

	fn is_udmf_map(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path)
			.map(|handle| handle.is_dir() && handle.contains("TEXTMAP"))
	}

	fn has_zscript(&self, path: impl AsRef<Path>) -> Option<bool> {
		lazy_static! {
			static ref RGX_ZSCRIPT: Regex = Regex::new(r"^(?i)zscript")
				.expect("Failed to evaluate `ImpureVfs::has_zscript::RGX_ZSCRIPT`.");
		};

		self.lookup(path)
			.map(|handle| handle.is_dir() && handle.contains_regex(&RGX_ZSCRIPT))
	}

	fn has_edfroot(&self, path: impl AsRef<Path>) -> Option<bool> {
		lazy_static! {
			static ref RGX_EDFROOT: Regex = Regex::new(r"^(?i)edfroot")
				.expect("Failed to evaluate `ImpureVfs::has_edfroot::RGX_EDFROOT`.");
		};

		self.lookup(path)
			.map(|handle| handle.is_dir() && handle.contains_regex(&RGX_EDFROOT))
	}

	fn has_decorate(&self, path: impl AsRef<Path>) -> Option<bool> {
		lazy_static! {
			static ref RGX_DECORATE: Regex = Regex::new(r"^(?i)decorate")
				.expect("Failed to evaluate `ImpureVfs::has_decorate::RGX_DECORATE`.");
		};

		self.lookup(path)
			.map(|handle| handle.is_dir() && handle.contains_regex(&RGX_DECORATE))
	}

	fn gamedata_kind(&self, uuid: &str) -> GameDataKind {
		fn check_path(path: &Path) -> std::option::Option<GameDataKind> {
			if path.has_gzdoom_extension() {
				return Some(GameDataKind::ZDoom);
			}

			if path.has_eternity_extension() {
				return Some(GameDataKind::Eternity);
			}

			if path.has_wad_extension() {
				return Some(GameDataKind::Wad);
			}

			None
		}

		let entry = &self
			.entries
			.iter()
			.find(|c| c.file_name() == uuid)
			.expect("Invalid UUID passed to `ImpureVfs::gamedata_kind`.");

		match &entry.kind {
			EntryKind::Leaf { .. } => {
				return GameDataKind::File;
			}
			EntryKind::Directory => {
				let real_path = self
					.real_paths
					.get(uuid)
					.expect("Invalid UUID passed to `ImpureVfs::gamedata_kind`.");

				for child in self.children_of(entry) {
					if !child.file_name().eq_ignore_ascii_case("meta.toml") {
						continue;
					}

					if child.is_dir() {
						continue;
					}

					let string = match child.read_str() {
						Ok(s) => s,
						Err(err) => {
							warn!(
								"Invalid meta.toml file under '{}': {}",
								entry.path_str(),
								err
							);
							continue;
						}
					};

					let mline = match string.lines().find(|l| l.starts_with("manifest = ")) {
						Some(l) => l,
						None => {
							warn!("Impure package '{}' defines no manifest.", entry.path_str());
							continue;
						}
					};

					let pb: PathBuf = match toml::from_str(mline) {
						Ok(p) => p,
						Err(err) => {
							warn!(
								"Failed to convert manifest string to path: '{}' ('{}')
								Error: {}",
								mline,
								entry.path_str(),
								err
							);
							continue;
						}
					};

					return GameDataKind::Impure { manifest: pb };
				}

				if let Some(kind) = check_path(real_path) {
					return kind;
				}
			}
		};

		// Either the user has done something unexpected
		// or we just need more heuristics I haven't come up with yet
		unreachable!();
	}

	fn parse_gamedata_meta(
		&self,
		path: impl AsRef<Path>,
	) -> Result<GameDataMeta, Box<dyn std::error::Error>> {
		let text = self.read_str(path.as_ref())?;
		let ret: GameDataMeta = toml::from_str(text)?;
		Ok(ret)
	}

	fn window_icon_from_file(&self, path: impl AsRef<Path>) -> Option<winit::window::Icon> {
		let bytes = match self.read(path) {
			Ok(b) => b,
			Err(err) => {
				error!("Failed to read engine icon image bytes: {}", err);
				return None;
			}
		};

		let icon = match image::load_from_memory(bytes) {
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

	fn ccmd_file(&self, path: PathBuf) -> String {
		let handle = match self.lookup(&path) {
			Some(e) => e,
			None => {
				return "Nothing exists at that path.".to_string();
			}
		};

		if !handle.is_dir() {
			return format!(
				"{}\r\n\tSize: {}B",
				handle.file_name(),
				handle.read().unwrap().len()
			);
		}

		let count = handle.count();
		let mut ret = String::with_capacity(count * 32);

		for child in handle.children() {
			match write!(ret, "\r\n\t{}", child.file_name()) {
				Ok(()) => {}
				Err(err) => {
					warn!(
						"Failed to write an output line for ccmd.: `file`
						Error: {}",
						err
					);
				}
			}

			if child.is_dir() {
				ret.push('/');
			}
		}

		format!("Files under \"{}\" ({}): {}", path.display(), count, ret)
	}
}

pub trait ImpureVfsHandle {
	/// Check if a directory node has a `meta.toml` leaf (case-insensitive) in it.
	/// Unconditionally returns false if the handle's entry is, itself, a leaf node.
	#[must_use]
	fn is_impure_package(&self) -> bool;
	/// Check if this is a directory with a leaf node named `TEXTMAP`.
	/// Unconditionally returns false if the handle's entry is, itself, a leaf node.
	#[must_use]
	fn is_udmf_map(&self) -> bool;
	/// Check if a directory node has a `decorate` file (case-insensitive) in it.
	/// Unconditionally returns false if the handle's entry is, itself, a leaf node.
	#[must_use]
	fn has_decorate(&self) -> bool;
	/// Check if a directory node has a `zscript` file (case-insensitive) in it.
	/// Unconditionally returns false if the handle's entry is, itself, a leaf node.
	#[must_use]
	fn has_zscript(&self) -> bool;
	/// Check if a directory node has an `edfroot` file (case-insensitive) in it.
	/// Unconditionally returns false if the handle's entry is, itself, a leaf node.
	#[must_use]
	fn has_edfroot(&self) -> bool;
}

impl ImpureVfsHandle for Handle<'_, '_> {
	fn is_impure_package(&self) -> bool {
		lazy_static! {
			static ref RGX_METATOML: Regex = Regex::new(r"^(?i)meta\.toml")
				.expect("Failed to evaluate `ImpureVfs::is_impure_package::RGX_METATOML`.");
		};

		self.is_dir() && self.contains_regex(&RGX_METATOML)
	}

	fn is_udmf_map(&self) -> bool {
		self.contains("TEXTMAP")
	}

	fn has_decorate(&self) -> bool {
		lazy_static! {
			static ref RGX_DECORATE: Regex = Regex::new(r"^(?i)decorate")
				.expect("Failed to evaluate `ImpureVfs::has_decorate::RGX_DECORATE`.");
		};

		self.is_dir() && self.contains_regex(&RGX_DECORATE)
	}

	fn has_zscript(&self) -> bool {
		lazy_static! {
			static ref RGX_ZSCRIPT: Regex = Regex::new(r"^(?i)zscript")
				.expect("Failed to evaluate `ImpureVfs::has_zscript::RGX_ZSCRIPT`.");
		};

		self.is_dir() && self.contains_regex(&RGX_ZSCRIPT)
	}

	fn has_edfroot(&self) -> bool {
		lazy_static! {
			static ref RGX_EDFROOT: Regex = Regex::new(r"^(?i)edfroot")
				.expect("Failed to evaluate `ImpureVfs::has_edfroot::RGX_EDFROOT`.");
		};

		self.is_dir() && self.contains_regex(&RGX_EDFROOT)
	}
}

impl ZsFileSystem for Handle<'_, '_> {
	fn get_file(&mut self, filename: &str) -> Option<ZsFile> {
		let target = match self.lookup_nocase(filename) {
			Some(h) => h,
			None => {
				let full_path = self.virtual_path().join(filename);
				warn!("Failed to find ZScript file: {}", full_path.display());
				return None;
			}
		};

		if target.is_dir() {
			let full_path = self.virtual_path().join(filename);
			warn!(
				"Expected ZScript file, found directory: {}",
				full_path.display()
			);
			return None;
		}

		Some(ZsFile::new(filename.to_string(), target.copy().unwrap()))
	}

	fn get_files_no_ext(&mut self, filename: &str) -> Vec<ZsFile> {
		let mut ret = Vec::<ZsFile>::default();

		for child in self.children() {
			let mut noext = child.file_name().splitn(2, '.');

			let bytes = if child.is_leaf() {
				child.read().unwrap()
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
