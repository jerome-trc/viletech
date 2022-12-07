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
along with this program.  If not, see <http://www.gnu.org/licenses/>.

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

use crate::{lazy_regexset, utils::io::*, vfs::RGX_INVALIDMOUNTPATH, wad};

use super::{Entry, EntryKind, Error, VirtualFs};

impl VirtualFs {
	pub(super) fn mount_parallel(
		&mut self,
		mounts: &[(impl AsRef<Path>, impl AsRef<Path>)],
	) -> Vec<Result<(), Error>> {
		let results = Vec::<(usize, Result<(), Error>)>::with_capacity(mounts.len());
		let results = Mutex::new(results);

		let mounts: Vec<(usize, (&Path, &Path))> = mounts
			.iter()
			.map(|pair| (pair.0.as_ref(), pair.1.as_ref()))
			.enumerate()
			.collect();

		let output = Mutex::new(Vec::<(Vec<Entry>, String, PathBuf)>::default());

		let (_, root) = self
			.lookup_hash(Self::hash_path("/"))
			.expect("VFS root node is missing.");

		let root_hash = root.hash;

		mounts.par_iter().for_each(|tuple| {
			let pair = &tuple.1;

			let real_path = match pair.0.canonicalize() {
				Ok(c) => c,
				Err(err) => {
					warn!(
						"Failed to canonicalize real path: {}
						Error: {}",
						pair.0.display(),
						err
					);
					results
						.lock()
						.push((tuple.0, Err(Error::Canonicalization(err))));
					return;
				}
			};

			let mount_point = pair.1;

			// Don't let the caller mount symbolic links, etc.

			match Self::mount_supported(&real_path) {
				Ok(()) => {}
				Err(err) => {
					warn!(
						"Attempted to mount an unsupported file: {}
						Reason: {}",
						real_path.display(),
						err
					);
					results.lock().push((tuple.0, Err(err)));
					return;
				}
			};

			// Ensure mount point is valid UTF-8

			let mpoint_str = match mount_point.to_str() {
				Some(s) => s,
				None => {
					warn!(
						"Attempted to use a mount point that isn't valid Unicode ({})",
						mount_point.display()
					);
					results.lock().push((tuple.0, Err(Error::InvalidUtf8)));
					return;
				}
			};

			// Ensure mount point is only alphanumerics and underscores

			if RGX_INVALIDMOUNTPATH.is_match(mpoint_str) {
				warn!(
					"Attempted to use a mount point that isn't comprised \
					solely of alphanumerics, underscores, dashes, periods, \
					and forward slashes. ({})",
					mount_point.display()
				);
				results
					.lock()
					.push((tuple.0, Err(Error::InvalidMountPoint)));
				return;
			}

			// Ensure nothing already exists at end of mount point

			if self.exists(mount_point) {
				results.lock().push((tuple.0, Err(Error::Remount)));
				return;
			}

			// All checks passed. Start recurring down real path

			let mut mpoint = PathBuf::new();

			if !mount_point.starts_with("/") {
				mpoint.push("/");
			}

			mpoint.push(mount_point);

			let res = if real_path.is_dir() {
				Self::mount_dir(&real_path, mpoint.clone(), root_hash)
			} else {
				let bytes = match fs::read(&real_path) {
					Ok(b) => b,
					Err(err) => {
						warn!(
							"Failed to read object for mounting: {}
							Error: {}",
							real_path.display(),
							err
						);

						results.lock().push((tuple.0, Err(Error::IoError(err))));
						return;
					}
				};

				Self::mount_file(bytes, mpoint.clone(), root_hash)
			};

			let new_entries = match res {
				Ok(e) => e,
				Err(err) => {
					warn!(
						"Failed to mount object: {}
						Error: {}",
						real_path.display(),
						err
					);
					return;
				}
			};

			info!(
				"Mounted: \"{}\" -> \"{}\".",
				real_path.display(),
				mpoint.display()
			);

			output
				.lock()
				.push((new_entries, mpoint.to_str().unwrap().to_owned(), real_path));
			results.lock().push((tuple.0, Ok(())));
		});

		let mut output = output.into_inner();

		for mut troika in output.drain(..) {
			self.entries.append(&mut troika.0);
			troika.1.remove(0); // Take off preceding root backslash
			self.real_paths.insert(troika.1, troika.2);
		}

		let mut results = results.into_inner();
		let mut ret = Vec::<Result<(), Error>>::with_capacity(results.len());

		while !results.is_empty() {
			let mut i = 0;

			while i < results.len() {
				if results[i].0 == ret.len() {
					ret.push(results.swap_remove(i).1);
				} else {
					i += 1;
				}
			}
		}

		debug_assert!(ret.len() == mounts.len());

		ret
	}

	/// Forwards files of an as-yet unknown kind to the right mounting function.
	pub(super) fn mount_file(
		bytes: Vec<u8>,
		mut virt_path: PathBuf,
		parent_hash: u64,
	) -> Result<Vec<Entry>, Error> {
		match is_valid_wad(&bytes[..], bytes.len().try_into().unwrap()) {
			Ok(b) => {
				if b {
					// If this WAD was nested in another archive,
					// it will need to have its extension taken off
					virt_path.set_extension("");
					return Self::mount_wad(bytes, virt_path, parent_hash);
				}
			}
			Err(err) => {
				warn!(
					"Failed to determine if file is a WAD: {}
					Error: {}",
					virt_path.display(),
					err
				);
				return Err(Error::IoError(err));
			}
		};

		if is_zip(&bytes) {
			// If this zip file was nested in another archive,
			// it will need to have its extension taken off
			virt_path.set_extension("");
			return Self::mount_zip(bytes, virt_path, parent_hash);
		}

		// This isn't any kind of archive. Mount whatever it may be

		Ok(vec![Entry::new_leaf(virt_path, parent_hash, bytes)])
	}

	fn mount_zip(
		bytes: Vec<u8>,
		virt_path: PathBuf,
		parent_hash: u64,
	) -> Result<Vec<Entry>, Error> {
		let cursor = Cursor::new(&bytes);
		let mut zip = ZipArchive::new(cursor).map_err(Error::ZipError)?;
		let mut ret = vec![Entry::new_dir(virt_path.clone(), parent_hash)];

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
						"Skipping malformed entry in zip archive: {}
						Error: {}",
						virt_path.display(),
						err
					);
					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(p) => p,
				None => {
					warn!(
						"A zip file entry contains an unsafe path at index: {}
						Zip file virtual path: {}",
						i,
						virt_path.display()
					);
					continue;
				}
			};

			let mut vpath = virt_path.clone();
			vpath.push(zfpath);
			let parent_path = vpath.parent().unwrap();
			let parent = ret.iter().find(|e| e.path.as_path() == parent_path);
			ret.push(Entry::new_dir(vpath, parent.unwrap().hash));
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
						"Skipping malformed entry in zip archive: {}
						Error: {}",
						virt_path.display(),
						err
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
							"Failed to read all bytes of zip file entry: {}
							Zip file virtual path: {}",
							zfile.enclosed_name().unwrap().display(),
							virt_path.display()
						);
						continue;
					}
				}
				Err(err) => {
					warn!(
						"Failed to read zip file entry: {}
						Zip file virtual path: {}
						Error: {}",
						zfile.enclosed_name().unwrap().display(),
						virt_path.display(),
						err
					);
					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(p) => p,
				None => {
					warn!(
						"A zip file entry contains an unsafe path at index: {}
						Zip file virtual path: {}",
						i,
						virt_path.display()
					);
					continue;
				}
			};

			let mut vpath = virt_path.clone();
			vpath.push(zfpath);
			let parent_path = vpath.parent().unwrap();
			let parent = ret.iter().find(|e| e.path.as_path() == parent_path);
			ret.push(Entry::new_leaf(vpath, parent.unwrap().hash, bytes));
		}

		ret[1..].sort_by(Entry::cmp_name);

		Ok(ret)
	}

	fn mount_wad(
		bytes: Vec<u8>,
		virt_path: PathBuf,
		parent_hash: u64,
	) -> Result<Vec<Entry>, Error> {
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

		let wad = wad::parse_wad(bytes).map_err(Error::WadError)?;
		let mut dissolution = wad.dissolve();

		let mut ret = vec![Entry::new_dir(virt_path.clone(), parent_hash)];
		let this_hash = ret.last().unwrap().hash;

		let mut mapfold: Option<Entry> = None;

		for (ebytes, name) in dissolution.drain(..) {
			let mut vpath = virt_path.clone();
			vpath.push(&name);

			if lazy_regexset!(r"^MAP[0-9]{2}$", r"^E[0-9]M[0-9]$", r"^HUBMAP$").is_match(&name) {
				if let Some(entry) = mapfold.take() {
					ret.push(entry);
				}

				mapfold = Some(Entry::new_dir(vpath, this_hash));
				continue;
			}

			let dup_pos = ret.iter().position(|entry| entry.file_name() == name);

			match dup_pos {
				None => {}
				Some(pos) => {
					let entry = ret.remove(pos);

					match entry.kind {
						EntryKind::Binary(bytes) => {
							let mut svpath0 = vpath.clone();
							svpath0.push("000");

							let mut svpath1 = vpath.clone();
							svpath1.push("001");

							let new_folder = Entry::new_dir(vpath, this_hash);
							let new_folder_hash = new_folder.hash;
							ret.push(new_folder);

							ret.push(Entry::new_leaf(svpath0, new_folder_hash, bytes));
							ret.push(Entry::new_leaf(svpath1, new_folder_hash, ebytes));
						}
						EntryKind::String(string) => {
							let mut svpath0 = vpath.clone();
							svpath0.push("000");

							let mut svpath1 = vpath.clone();
							svpath1.push("001");

							let new_folder = Entry::new_dir(vpath, this_hash);
							let new_folder_hash = new_folder.hash;
							ret.push(new_folder);

							ret.push(Entry::new_leaf(
								svpath0,
								new_folder_hash,
								string.into_bytes(),
							));
							ret.push(Entry::new_leaf(svpath1, new_folder_hash, ebytes));
						}
						EntryKind::Directory => {
							let count = ret.iter().filter(|e| e.parent_hash == entry.hash).count();

							let mut svpath = vpath.clone();
							svpath.push(format!("{:03}", count));

							ret.push(Entry::new_leaf(svpath, entry.hash, ebytes));
						}
					}

					continue;
				}
			}

			let pop_map = match &mut mapfold {
				Some(folder) => {
					let mut is_map_part = false;

					for lmpname in MAP_COMPONENTS {
						if name.eq_ignore_ascii_case(lmpname) {
							is_map_part = true;
							break;
						}
					}

					if is_map_part {
						ret.push(Entry::new_leaf(vpath, folder.hash, ebytes));
						continue;
					} else {
						true
					}
				}
				None => {
					ret.push(Entry::new_leaf(vpath, parent_hash, ebytes));
					continue;
				}
			};

			if pop_map {
				ret.push(mapfold.take().unwrap());
			}
		}

		if mapfold.is_some() {
			ret.push(mapfold.take().unwrap());
		}

		Ok(ret)
	}

	pub(super) fn mount_dir(
		real_path: &Path,
		virt_path: PathBuf,
		parent_hash: u64,
	) -> Result<Vec<Entry>, Error> {
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

		ret.push(Entry::new_dir(virt_path.clone(), parent_hash));
		let this_hash = ret.last().unwrap().hash;

		for entry in read_dir {
			let ftype = match entry.file_type() {
				Ok(ft) => ft,
				Err(err) => {
					warn!(
						"Skipping mounting dir. entry of unknown type: {}
						File type acquiry error: {}",
						entry.path().display(),
						err
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
						"Directory entry with invalid UTF-8 in file name will \
						not be mounted: {}",
						entry_path.display()
					);
					continue;
				}
			};

			let mut vpath = virt_path.clone();
			vpath.push(fname);

			let res = if ftype.is_dir() {
				Self::mount_dir(&entry_path, vpath, this_hash)
			} else {
				let bytes = match fs::read(&entry_path) {
					Ok(b) => b,
					Err(err) => {
						warn!(
							"Failed to read object for mounting: {}
							Error: {}",
							entry_path.display(),
							err
						);

						return Err(Error::IoError(err));
					}
				};

				Self::mount_file(bytes, vpath, this_hash)
			};

			match res {
				Ok(mut e) => {
					ret.append(&mut e);
				}
				Err(err) => {
					warn!(
						"Failed to mount directory entry: {}
						Error: {}",
						entry_path.display(),
						err
					);
					continue;
				}
			}
		}

		ret[1..].sort_by(Entry::cmp_name);

		Ok(ret)
	}
}
