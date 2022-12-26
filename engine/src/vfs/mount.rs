//! Internal implementation details: anything related to mounting.

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
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use std::{
	fs,
	io::{self, Cursor},
	path::{Path, PathBuf},
};

use log::{info, warn};
use parking_lot::Mutex;
use rayon::prelude::*;
use zip::ZipArchive;

use crate::{utils::io::*, wad};

use super::{entry::PathHash, Entry, Error, VirtualFs, RGX_INVALIDMOUNTPATH};

impl VirtualFs {
	// Note to self: in a 2-path tuple, `::0` is real and `::1` is virtual

	#[must_use]
	pub(super) fn mount_serial(
		&mut self,
		mounts: &[(impl AsRef<Path>, impl AsRef<Path>)],
	) -> Vec<Result<(), Error>> {
		let mut ret = Vec::with_capacity(mounts.len());

		for (real_path, mount_path) in mounts {
			let real_path = match real_path.as_ref().canonicalize() {
				Ok(canon) => canon,
				Err(err) => {
					ret.push(Err(Error::Canonicalization(err)));
					continue;
				}
			};

			let mount_path = mount_path.as_ref();

			if !real_path.exists() {
				ret.push(Err(Error::NonExistentFile(real_path)));
				continue;
			}

			if real_path.is_symlink() {
				ret.push(Err(Error::SymlinkMount));
				continue;
			}

			// Ensure mount point is valid UTF-8

			let mount_point_str = match mount_path.to_str() {
				Some(s) => s,
				None => {
					ret.push(Err(Error::InvalidUtf8));
					continue;
				}
			};

			// Ensure mount point is only alphanumerics and underscores

			if RGX_INVALIDMOUNTPATH.is_match(mount_point_str) {
				ret.push(Err(Error::InvalidMountPoint));
				continue;
			}

			// Ensure mount point path has a parent path

			let mount_point_parent = match mount_path.parent() {
				Some(p) => p,
				None => {
					ret.push(Err(Error::ParentlessMountPoint));
					continue;
				}
			};

			// Ensure mount point parent exists

			if !self.exists(mount_point_parent) {
				ret.push(Err(Error::NonExistentEntry(mount_point_parent.to_owned())));
				continue;
			}

			// Ensure nothing already exists at end of mount point

			if self.exists(mount_path) {
				ret.push(Err(Error::Remount));
				continue;
			}

			// All checks passed. Start recurring down real path

			let mut mount_point = PathBuf::new();

			if !mount_path.starts_with("/") {
				mount_point.push("/");
			}

			mount_point.push(mount_path);

			let res = if real_path.is_dir() {
				Self::mount_dir(&real_path, mount_point.clone())
			} else {
				let bytes = match fs::read(&real_path) {
					Ok(b) => b,
					Err(err) => {
						ret.push(Err(Error::IoError(err)));
						continue;
					}
				};

				Self::mount_file(bytes, mount_point.clone())
			};

			let new_entries = match res {
				Ok(e) => e,
				Err(err) => {
					ret.push(Err(err));
					continue;
				}
			};

			info!(
				"Mounted: \"{}\" -> \"{}\".",
				real_path.display(),
				mount_point.display()
			);

			self.real_paths
				.insert(real_path, PathHash::new(&mount_point));

			for new_entry in new_entries {
				let displaced = self
					.entries
					.insert(PathHash::new(&new_entry.path), new_entry);

				debug_assert!(
					displaced.is_none(),
					"A VFS serial mount displaced entry: {}",
					displaced.unwrap().path.display()
				);
			}

			ret.push(Ok(()));
		}

		debug_assert!(ret.len() == mounts.len());

		ret
	}

	#[must_use]
	pub(super) fn mount_parallel(
		&mut self,
		mounts: &[(impl AsRef<Path>, impl AsRef<Path>)],
	) -> Vec<Result<(), Error>> {
		enum Output {
			Uninit,
			Ok {
				new_entries: Vec<Entry>,
				mount_point: PathBuf,
				real_path: PathBuf,
			},
			Err(Error),
		}

		let mut results = Vec::with_capacity(mounts.len());

		for _ in 0..mounts.len() {
			results.push(Output::Uninit);
		}

		let results = Mutex::new(results);

		let mounts: Vec<(&Path, &Path)> = mounts
			.iter()
			.map(|pair| (pair.0.as_ref(), pair.1.as_ref()))
			.collect();

		mounts
			.par_iter()
			.enumerate()
			.for_each(|(index, (real_path, mount_path))| {
				let real_path = match real_path.canonicalize() {
					Ok(canon) => canon,
					Err(err) => {
						results.lock()[index] = Output::Err(Error::Canonicalization(err));
						return;
					}
				};

				if !real_path.exists() {
					results.lock()[index] = Output::Err(Error::NonExistentFile(real_path));
					return;
				}

				if real_path.is_symlink() {
					results.lock()[index] = Output::Err(Error::SymlinkMount);
					return;
				}

				// Ensure mount point is valid UTF-8

				let mount_point_str = match mount_path.to_str() {
					Some(s) => s,
					None => {
						results.lock()[index] = Output::Err(Error::InvalidUtf8);
						return;
					}
				};

				// Ensure mount point is only alphanumerics and underscores

				if RGX_INVALIDMOUNTPATH.is_match(mount_point_str) {
					results.lock()[index] = Output::Err(Error::InvalidMountPoint);
					return;
				}

				// Ensure mount point path has a parent path

				let mount_point_parent = match mount_path.parent() {
					Some(p) => p,
					None => {
						results.lock()[index] = Output::Err(Error::ParentlessMountPoint);
						return;
					}
				};

				// Ensure mount point parent exists

				if !self.exists(mount_point_parent) {
					results.lock()[index] =
						Output::Err(Error::NonExistentEntry(mount_point_parent.to_owned()));
					return;
				}

				// Ensure nothing already exists at end of mount point

				if self.exists(mount_path) {
					results.lock()[index] = Output::Err(Error::Remount);
					return;
				}

				// All checks passed. Start recurring down real path

				let mut mount_point = PathBuf::new();

				if !mount_path.starts_with("/") {
					mount_point.push("/");
				}

				mount_point.push(mount_path);

				let res = if real_path.is_dir() {
					Self::mount_dir(&real_path, mount_point.clone())
				} else {
					let bytes = match fs::read(&real_path) {
						Ok(b) => b,
						Err(err) => {
							results.lock()[index] = Output::Err(Error::IoError(err));
							return;
						}
					};

					Self::mount_file(bytes, mount_point.clone())
				};

				let new_entries = match res {
					Ok(e) => e,
					Err(err) => {
						results.lock()[index] = Output::Err(err);
						return;
					}
				};

				info!(
					"Mounted: \"{}\" -> \"{}\".",
					real_path.display(),
					mount_point.display()
				);

				results.lock()[index] = Output::Ok {
					new_entries,
					mount_point,
					real_path,
				};
			});

		let ret: Vec<Result<(), Error>> = results
			.into_inner()
			.into_iter()
			.map(|out| match out {
				Output::Uninit => {
					unreachable!("A VFS parallel mount result was left uninitialized.");
				}
				Output::Ok {
					new_entries,
					mount_point,
					real_path,
				} => {
					self.real_paths
						.insert(real_path, PathHash::new(mount_point));

					for new_entry in new_entries {
						let displaced = self
							.entries
							.insert(PathHash::new(&new_entry.path), new_entry);

						debug_assert!(
							displaced.is_none(),
							"A VFS parallel mount displaced entry: {}",
							displaced.unwrap().path.display()
						);
					}

					Ok(())
				}
				Output::Err(err) => Err(err),
			})
			.collect();

		debug_assert!(ret.len() == mounts.len());

		ret
	}

	/// Forwards files of an as-yet unknown kind to the right mounting function.
	fn mount_file(bytes: Vec<u8>, mut virt_path: PathBuf) -> Result<Vec<Entry>, Error> {
		match is_valid_wad(&bytes[..], bytes.len().try_into().unwrap()) {
			Ok(b) => {
				if b {
					// If this WAD was nested in another archive,
					// it will need to have its extension taken off
					virt_path.set_extension("");
					return Self::mount_wad(bytes, virt_path);
				}
			}
			Err(err) => {
				warn!(
					"Failed to determine if file is a WAD: {}\r\n\
					Error: {err}",
					virt_path.display(),
				);
				return Err(Error::IoError(err));
			}
		};

		if is_zip(&bytes) {
			// If this zip file was nested in another archive,
			// it will need to have its extension taken off
			virt_path.set_extension("");
			return Self::mount_zip(bytes, virt_path);
		}

		// This isn't any kind of archive. Mount whatever it may be

		Ok(vec![Entry::new_leaf(virt_path, bytes)])
	}

	fn mount_zip(bytes: Vec<u8>, virt_path: PathBuf) -> Result<Vec<Entry>, Error> {
		let cursor = Cursor::new(&bytes);
		let mut zip = ZipArchive::new(cursor).map_err(Error::ZipError)?;
		let mut ret = vec![Entry::new_dir(virt_path.clone())];

		// First pass creates a directory structure

		for i in 0..zip.len() {
			let zfile = match zip.by_index(i) {
				Ok(z) => {
					if !z.is_dir() {
						continue;
					} else {
						z
					}
				}
				Err(err) => {
					warn!(
						"Skipping malformed entry in zip archive: {}\r\n\
						Error: {err}",
						virt_path.display(),
					);
					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(p) => p,
				None => {
					warn!(
						"A zip file entry contains an unsafe path at index: {i}\r\n
						Zip file virtual path: {}",
						virt_path.display()
					);
					continue;
				}
			};

			let mut vpath = virt_path.clone();
			vpath.push(zfpath);
			ret.push(Entry::new_dir(vpath));
		}

		// Second pass covers leaf nodes

		for i in 0..zip.len() {
			let mut zfile = match zip.by_index(i) {
				Ok(z) => {
					if z.is_dir() {
						continue;
					} else {
						z
					}
				}
				Err(err) => {
					warn!(
						"Skipping malformed entry in zip archive: {}\r\n\
						Error: {err}",
						virt_path.display(),
					);
					continue;
				}
			};

			let zfsize = zfile.size();
			let mut bytes = Vec::<u8>::with_capacity(zfsize.try_into().unwrap());

			match io::copy(&mut zfile, &mut bytes) {
				Ok(count) => {
					if count != zfsize {
						warn!(
							"Failed to read all bytes of zip file entry: {}\r\n\
							Zip file virtual path: {}",
							zfile.enclosed_name().unwrap().display(),
							virt_path.display()
						);
						continue;
					}
				}
				Err(err) => {
					warn!(
						"Failed to read zip file entry: {}\r\nZip file virtual path: {}\r\n\
						Error: {err}",
						zfile.enclosed_name().unwrap().display(),
						virt_path.display(),
					);
					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(p) => p,
				None => {
					warn!(
						"A zip file entry contains an unsafe path at index: {i}\r\n\
						Zip file virtual path: {}",
						virt_path.display()
					);
					continue;
				}
			};

			let mut vpath = virt_path.clone();
			vpath.push(zfpath);
			ret.push(Entry::new_leaf(vpath, bytes));
		}

		ret[1..].par_sort_by(Entry::cmp_name);

		Ok(ret)
	}

	fn mount_wad(bytes: Vec<u8>, virt_path: PathBuf) -> Result<Vec<Entry>, Error> {
		#[rustfmt::skip]
		const MAP_COMPONENTS: &[&str] = &[
			"blockmap",
			"linedefs",
			"nodes",
			"reject",
			"sectors",
			"segs",
			"sidedefs",
			"ssectors",
			"things",
			"vertexes",
			// UDMF
			"behavior",
			"dialogue",
			"scripts",
			"textmap",
			"znodes",
			// Note: ENDMAP gets filtered out, since there's no need to keep it
		];

		#[must_use]
		fn is_map_component(name: &str) -> bool {
			MAP_COMPONENTS.iter().any(|s| s.eq_ignore_ascii_case(name))
		}

		#[must_use]
		fn is_first_map_component(name: &str) -> bool {
			name.eq_ignore_ascii_case("things") || name.eq_ignore_ascii_case("textmap")
		}

		let wad = wad::parse_wad(bytes).map_err(Error::WadError)?;
		let mut ret = Vec::with_capacity(wad.len());
		let mut dissolution = wad.dissolve();
		ret.push(Entry::new_dir(virt_path.clone()));
		let mut index = 0_usize;
		let mut mapfold: Option<usize> = None;

		for (bytes, name) in dissolution.drain(..) {
			index += 1;

			// Is this WAD entry the first component in a map grouping?
			// If so, the previous WAD entry was a map marker, and needs to be
			// treated like a directory. Future WAD entries until the next map
			// marker or non-map component will get a child path to [index - 1]
			if is_first_map_component(&name) && ret[index - 1].is_empty() {
				let prev = ret.pop().unwrap();
				ret.push(Entry::new_dir(prev.path));
				mapfold = Some(index - 1);
			} else if !is_map_component(name.as_str()) {
				mapfold = None;
			}

			let child_path = if let Some(entry_idx) = mapfold {
				// virt_path currently: `/mount_point`
				let mut cpath = virt_path.join(ret[entry_idx].file_name());
				// cpath currently: `/mount_point/MAP01`
				cpath.push(&name);
				// cpath currently: `/mount_point/MAP01/THINGS`
				cpath
			} else {
				virt_path.join(&name)
			};

			// What if a WAD contains two entries with the same name?
			// (e.g. DOOM2.WAD has two identical `SW18_7` entries)
			// In this case, the last entry clobbers the previous ones

			if let Some(pos) = ret.iter().position(|e| e.path == child_path) {
				ret.remove(pos);
			}

			ret.push(Entry::new_leaf(child_path, bytes));
		}

		Ok(ret)
	}

	fn mount_dir(real_path: &Path, virt_path: PathBuf) -> Result<Vec<Entry>, Error> {
		let mut ret = Vec::<Entry>::default();

		// Check under this directory for other files/directories/archives

		let read_dir = match fs::read_dir(real_path) {
			Ok(r) => r.filter_map(|res| match res {
				Ok(r) => Some(r),
				Err(_) => None,
			}),
			Err(err) => {
				return Err(Error::DirectoryRead(err));
			}
		};

		ret.push(Entry::new_dir(virt_path.clone()));

		for entry in read_dir {
			let ftype = match entry.file_type() {
				Ok(ft) => ft,
				Err(err) => {
					warn!(
						"Skipping mounting dir. entry of unknown type: {}\r\n\
						File type acquiry error: {err}",
						entry.path().display(),
					);
					continue;
				}
			};

			if ftype.is_symlink() {
				continue;
			}

			let entry_path = entry.path();

			let fname = entry.file_name();
			let fname = match fname.to_str() {
				Some(f) => f,
				None => {
					warn!(
						"Directory entry with invalid UTF-8 in file name \
						will not be mounted: {}",
						entry_path.display()
					);
					continue;
				}
			};

			let mut vpath = virt_path.clone();
			vpath.push(fname);

			let res = if ftype.is_dir() {
				Self::mount_dir(&entry_path, vpath)
			} else {
				let bytes = match fs::read(&entry_path) {
					Ok(b) => b,
					Err(err) => {
						warn!(
							"Failed to read object for mounting: {}\r\n\
							Error: {err}",
							entry_path.display(),
						);

						return Err(Error::IoError(err));
					}
				};

				Self::mount_file(bytes, vpath)
			};

			match res {
				Ok(mut e) => {
					ret.append(&mut e);
				}
				Err(err) => {
					warn!(
						"Failed to mount directory entry: {}\r\n\
						Error: {err}",
						entry_path.display(),
					);
					continue;
				}
			}
		}

		ret[1..].par_sort_by(Entry::cmp_name);

		Ok(ret)
	}
}
