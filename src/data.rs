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

use super::lua::ImpureLua;
use crate::{vfs::{ImpureVfs, VirtualFs}, ecs::Blueprint, game::{Species, DamageType}};
use core::fmt;
use log::{error, warn};
use mlua::prelude::*;
use std::{
	env,
	error::Error,
	fs, io,
	path::{Path, PathBuf}, collections::HashMap,
};

/// General-purpose semantic versioning triplet.
pub struct SemVer {
	pub major: u16,
	pub minor: u16,
	pub patch: u16,
}

pub struct VersionedId {
	// Note to reader: probably not going to go to the same extent as npm
	// semantic versioning but there should be some versioning tied to this
	pub uuid: String,
}

#[derive(PartialEq)]
pub enum PackageType {
	None,
	Wad,
	Impure,
	GzDoom,
	Eternity,
}

pub struct PkgMeta {
	pub uuid: String,
	pub version: SemVer,
	pub name: String,
	pub desc: String,
	pub author: String,
	pub copyright: String,
	pub link: String,
	pub directory: PathBuf,
	pub mount_point: String,
	pub dependencies: Vec<VersionedId>,
	/// Incompatibilities are "soft";
	/// the user is warned when trying to mingle incompatible packages
	/// but can still proceed as normal.
	pub incompatibilities: Vec<VersionedId>,
}

#[derive(Debug)]
pub enum PkgMetaParseError<'p> {
	FileReadError(&'p Path, io::Error),
	LuaEvalError(&'p Path, mlua::Error),
}

impl<'p> Error for PkgMetaParseError<'p> {}

impl<'p> fmt::Display for PkgMetaParseError<'p> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		return match self {
			Self::FileReadError(path, err) => {
				write!(f, "Failed to read path: {}\nError: {}", path.display(), err)
			}
			Self::LuaEvalError(path, err) => {
				write!(
					f,
					"Failed to evaluate package metadata at path: {}\nError: {}",
					path.display(),
					err
				)
			}
		};
	}
}

pub fn parse_dependencies(pkgmeta: &LuaTable, path: &str) -> Vec<VersionedId> {
	let depends: LuaTable = match pkgmeta.get::<_, LuaTable>("depends") {
		Ok(tbl) => tbl,
		Err(_) => {
			return Vec::default();
		}
	};

	let mut ret = Vec::<VersionedId>::default();

	for kvp in depends.pairs::<i32, String>() {
		if kvp.is_err() {
			warn!("Skipping invalid dependency entry in: {}", path);
			continue;
		}

		let (key, val) = kvp.expect("Failed to destructure a Lua key-value pair.");

		let pair: Vec<&str> = val.split('@').collect();

		match pair.len() {
			1 => {
				ret.push(VersionedId {
					uuid: pair[0].to_string(),
				});
			}
			_ => {
				warn!("Skipping invalid dependency entry {} in: {}", key, path);
				continue;
			}
		}
	}

	// TODO: Versioning of dependency specifications

	ret
}

pub fn parse_incompatibilities(pkgmeta: &LuaTable, path: &str) -> Vec<VersionedId> {
	let incompats: LuaTable = match pkgmeta.get::<_, LuaTable>("incompat") {
		Ok(tbl) => tbl,
		Err(_) => {
			return Vec::default();
		}
	};

	let mut ret = Vec::<VersionedId>::default();

	for kvp in incompats.pairs::<i32, String>() {
		if kvp.is_err() {
			warn!("Skipping invalid incompatibility entry in: {}", path);
			continue;
		}

		let (key, val) = kvp.expect("Failed to destructure a Lua key-value pair.");

		let pair: Vec<&str> = val.split('@').collect();

		match pair.len() {
			1 => {
				ret.push(VersionedId {
					uuid: pair[0].to_string(),
				});
			}
			_ => {
				warn!(
					"Skipping invalid incompatibility entry {} in: {}",
					key, path
				);
				continue;
			}
		}
	}

	// TODO: Versioning of incompatibility specifications

	ret
}

pub fn mount_gamedata(vfs: &mut VirtualFs, lua: &Lua, path: PathBuf) {
	let entries = match fs::read_dir(path.as_path()) {
		Ok(entries) => entries,
		// On the dev build, this is a valid state for the dir. structure to
		// be in. If the requisite gamedata isn't found in the PWD instead,
		// *that* represents an engine failure state
		Err(err) => {
			if err.kind() != io::ErrorKind::NotFound {
				error!("Failed to read gamedata directory: {}", err);
			}

			return;
		}
	};

	for entry in entries.filter_map(|e| match e {
		Ok(de) => Some(de),
		Err(err) => {
			error!(
				"Error encountered while reading gamedata directory: {}",
				err
			);
			None
		}
	}) {
		match entry.metadata() {
			Ok(metadata) => {
				if metadata.is_symlink() {
					continue;
				}
			}
			Err(err) => {
				error!(
					"Failed to retrieve metadata for gamedata directory entry: {:?}\nError: {}",
					entry.file_name(),
					err
				);
				continue;
			}
		};

		let pathbuf = entry.path();

		let pstr = if let Some(p) = pathbuf.as_path().to_str() {
			p
		} else {
			warn!(
				"Gamedata entry path is invalid, and will be skipped: {:?}",
				pathbuf.as_path()
			);
			continue;
		};

		let is_impure = vfs.package_type(pathbuf.as_path()) == PackageType::Impure;

		let mount_point = if let true = is_impure {
			let pmeta_path: PathBuf = [pathbuf.clone(), PathBuf::from("meta.lua")]
				.iter()
				.collect();

			let meta = match lua.parse_package_meta(pmeta_path.as_path()) {
				Ok(m) => m,
				Err(err) => {
					warn!(
						"Failed to parse metadata for package at path: {:?}\nError: {}",
						pathbuf, err
					);
					continue;
				}
			};

			meta.mount_point
		} else {
			let stem = match pathbuf.as_path().file_stem() {
				Some(s) => s,
				None => {
					warn!(
						"Gamedata entry path {:?} has no file name, 
						and will not be mounted.",
						pathbuf
					);
					continue;
				}
			};

			match stem.to_str() {
				Some(s) => s,
				None => {
					warn!(
						"Gamedata entry path {:?} contains invalid unicode,
						and will not be mounted.",
						pathbuf
					);
					continue;
				}
			}
			.to_owned()
		};

		match vfs.mount(pstr, &mount_point) {
			Ok(_) => {}
			Err(err) => {
				warn!(
					"Failed to mount gamedata entry to virtual file system: 
					{:?}\nError: {}",
					pathbuf.as_path(),
					err
				);
			}
		};
	}
}

/// Returns `None` if this platform is unsupported
/// or the home directory path is malformed.
pub fn get_userdata_path() -> Option<PathBuf> {
	let mut ret = match home::home_dir() {
		Some(hdir) => hdir,
		None => {
			return None;
		}
	};

	match env::consts::OS {
		"linux" => {
			ret.push(".config");
			ret.push("impure");
		}
		"windows" => {
			ret.push("impure");
		}
		_ => {
			return None;
		}
	}

	Some(ret)
}

pub fn mount_userdata(vfs: &mut VirtualFs) -> Result<(), io::Error> {
	let udata_path = match get_userdata_path() {
		Some(up) => up,
		None => {
			error!(
				"Failed to retrieve userdata path. 
				Home directory path is malformed, 
				or this platform is currently unsupported."
			);
			return Err(io::ErrorKind::Other.into());
		}
	};

	match fs::metadata(udata_path.as_path()) {
		Ok(metadata) => {
			if !metadata.is_dir() {
				match fs::remove_file(udata_path.as_path()) {
					Ok(_) => {}
					Err(rmerr) => {
						return Err(rmerr);
					}
				}

				match fs::create_dir(udata_path.as_path()) {
					Ok(_) => {}
					Err(cderr) => {
						return Err(cderr);
					}
				}
			}
		}
		Err(mtderr) => {
			if let std::io::ErrorKind::NotFound = mtderr.kind() {
				match fs::create_dir(udata_path.as_path()) {
					Ok(_) => {}
					Err(cderr) => {
						return Err(cderr);
					}
				}
			} else {
				return Err(mtderr);
			}
		}
	}

	let udata_pstr = match udata_path.to_str() {
		Some(s) => s,
		None => {
			error!("Failed to convert userdata path into a valid string.");
			return Err(io::ErrorKind::Other.into());
		}
	};

	match vfs.mount(udata_pstr, "/userdata") {
		Ok(_) => {}
		Err(err) => {
			return Err(io::Error::new(io::ErrorKind::Other, err));
		}
	};

	Ok(())
}

pub type AssetId = usize;

#[derive(Default)]
pub struct DataCore {
	/// Key structure:
	/// "package_uuid.domain.asset_key"
	/// Package UUID will either come from an Impure package metadata file,
	/// or from the archive/directory name minus the extension if it's not 
	/// Impure data (e.g. "DOOM2" from "DOOM2.WAD", "gzdoom" from "gzdoom.pk3").
	/// Domain will be something like "textures" or "blueprints".
	/// Asset key is derived from the file name
	/// Each value maps to an index in one of the asset vectors.
	asset_map: HashMap<String, AssetId>,
	/// e.g. if DOOM2 defines MAP01 and then my_house.wad is loaded after it and
	/// also defines a MAP01, the key "MAP01" will point to my_house.wad:MAP01.
	end_map: HashMap<String, AssetId>,

	language: Vec<String>,
	blueprints: Vec<Blueprint>,
	damage_types: Vec<DamageType>,
	species: Vec<Species>
}
