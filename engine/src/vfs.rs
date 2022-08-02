//! Abstraction over the OS file system for security and ease.
//! Inspired by PhysicsFS, but differs in that it owns every byte mounted,
//! for maximum-speed reading when organizing assets afterwards.

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

use lazy_static::lazy_static;
use log::{error, info, warn};
use parking_lot::Mutex;
use rayon::prelude::*;
use regex::{Regex, RegexSet};
use std::{
	fmt, fs,
	io::{self, Cursor},
	path::{Path, PathBuf},
};
use zip::{result::ZipError, ZipArchive};

use crate::{data::GameDataMeta, utils::{io::*, path::*, string::*}, wad, zsparse::*};

pub struct VirtualFs {
	root: Entry,
}

// Public interface.
impl VirtualFs {
	/// For each tuple of the given slice, `::0` should be the path to the real
	/// file/directory, and `::1` should be the desired "mount point".
	/// Returns a `Vec` parallel to `mounts` which contains `true` for each
	/// successful mount and `false` otherwise.
	pub fn mount(
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

		let output = Mutex::new(Vec::<Entry>::default());

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

			let mount_name = match mount_point.file_name().ok_or(Error::NonExistentEntry) {
				Ok(mn) => mn,
				Err(err) => {
					warn!(
						"Failed to get mount name from mount point: {}
						Error: {}",
						mount_point.display(),
						err
					);
					results.lock().push((tuple.0, Err(err)));
					return;
				}
			};

			let mount_name = match mount_name.to_str().ok_or(Error::InvalidUtf8) {
				Ok(mn) => mn,
				Err(err) => {
					warn!(
						"Failed to get mount name from mount point: {}
							Error: {}",
						mount_point.display(),
						err
					);
					results.lock().push((tuple.0, Err(err)));
					return;
				}
			};

			let res = if real_path.is_dir() {
				Self::mount_dir(&real_path, mount_name)
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

				Self::mount_file(bytes, mount_name)
			};

			let mut new_entry = match res {
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

			new_entry.sort();
			output.lock().push(new_entry);
			results.lock().push((tuple.0, Ok(())));

			info!(
				"Mounted: \"{}\" -> \"{}\".",
				real_path.display(),
				mount_point.display()
			);
		});

		let mut output = output.into_inner();

		for entry in output.drain(..) {
			self.root.children_mut().push(entry);
		}

		self.root.children_mut().sort_by(Entry::cmp_name);

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

		ret
	}

	pub fn mount_supported(path: impl AsRef<Path>) -> Result<(), Error> {
		let path = path.as_ref();

		if !path.exists() {
			return Err(Error::NonExistentFile);
		}

		if path.is_symlink() {
			return Err(Error::SymlinkMount);
		}

		if path.is_dir() {
			return Ok(());
		}

		Ok(())
	}

	pub fn exists(&self, path: impl AsRef<Path>) -> bool {
		self.lookup(path).is_some()
	}

	/// Returns `false` if nothing is at the given path.
	pub fn is_dir(&self, path: impl AsRef<Path>) -> bool {
		match self.lookup(path) {
			Some(entry) => entry.is_dir(),
			None => false,
		}
	}

	/// Returns [`Error::NonExistentEntry`] if there's nothing at the supplied path,
	/// or [`Error::Unreadable`] if attempting to read a directory.
	pub fn read(&self, path: impl AsRef<Path>) -> Result<&[u8], Error> {
		let entry = match self.lookup(path) {
			Some(e) => e,
			None => { return Err(Error::NonExistentEntry); }
		};

		match &entry.kind {
			EntryKind::Directory { .. } => Err(Error::Unreadable),
			EntryKind::Leaf { bytes } => Ok(&bytes[..]),
		}
	}

	/// Returns [`Error::InvalidUtf8`] if the contents at the path are not valid UTF-8.
	/// Otherwise acts like [`VirtualFs::read`].
	pub fn read_str(&self, path: impl AsRef<Path>) -> Result<&str, Error> {
		let bytes = self.read(path)?;

		match std::str::from_utf8(bytes) {
			Ok(ret) => Ok(ret),
			Err(_) => Err(Error::InvalidUtf8),
		}
	}

	/// Returns `None` if and only if nothing exists at the given path.
	pub fn lookup(&self, path: impl AsRef<Path>) -> Option<&Entry> {
		let p = path.as_ref();

		if p.is_empty() || p.is_root() {
			return Some(&self.root);
		}

		let mut iter = str_iter_from_path(p);

		let p = match iter.next() {
			Some(n) => {
				if n == "/" {
					match iter.next() {
						Some(next) => next,
						None => { return None; }
					}
				} else {
					n
				}
			}
			None => {
				return None;
			}
		};

		let children = self.root.children();

		for entry in children {
			if p != entry.name {
				continue;
			}

			return entry.lookup(iter);
		}

		None
	}

	/// Returns `Some(0)` if the given path is a leaf node.
	/// Returns `None` if and only if nothing exists at the given path.
	pub fn count(&self, path: impl AsRef<Path>) -> Option<usize> {
		let entry = self.lookup(path)?;

		if entry.is_leaf() {
			Some(0)
		} else {
			Some(entry.children().len())
		}
	}

	/// Returns `None` if and only if nothing exists at the given path.
	pub fn file_names<'s>(
		&'s self,
		path: impl AsRef<Path>,
	) -> Option<impl Iterator<Item = &'s String>> {
		let entry: &'s Entry = self.lookup(path)?;
		let closure = |c: &'s Entry| -> &'s String { &c.name };

		if entry.is_leaf() {
			let slice: &[Entry] = &[];
			return Some(slice.iter().map(closure));
		}

		let children = entry.children();
		Some(children.iter().map(closure))
	}

	/// Get the entries underneath a directory, as well as the number of entries.
	/// Precludes the need for 2 lookups to get both the iterator and count.
	/// If a path to a leaf node is given, an empty iterator is returned.
	/// Returns `None` if and only if nothing exists at the given path.
	pub fn itemize<'s>(
		&'s self,
		path: impl AsRef<Path>,
	) -> Option<(impl Iterator<Item = &'s Entry>, usize)> {
		let entry: &'s Entry = match self.lookup(path) {
			Some(e) => e,
			None => {
				return None;
			}
		};

		if entry.is_leaf() {
			let slice: &[Entry] = &[];
			return Some((slice.iter(), 0));
		}

		let children = entry.children();
		Some((children.iter(), children.len()))
	}
}

pub struct Entry {
	name: String,
	kind: EntryKind,
}

pub enum EntryKind {
	Leaf { bytes: Vec<u8> },
	Directory { children: Vec<Entry> },
}

impl Entry {
	pub fn get_name(&self) -> &str {
		&self.name
	}

	pub fn is_leaf(&self) -> bool {
		matches!(self.kind, EntryKind::Leaf { .. })
	}

	pub fn is_dir(&self) -> bool {
		matches!(self.kind, EntryKind::Directory { .. })
	}

	/// Note: non-recursive.
	/// Panics if used on a leaf node. Check to ensure it's a directory beforehand.
	pub fn contains(&self, name: &str) -> bool {
		for child in self.children() {
			if child.name == name {
				return true;
			}
		}

		false
	}

	/// Note: non-recursive.
	/// Panics if used on a leaf node. Check to ensure it's a directory beforehand.
	pub fn contains_regex(&self, regex: &Regex) -> bool {
		for child in self.children() {
			if regex.is_match(&child.name) {
				return true;
			}
		}

		false
	}

	/// Panics if used on a leaf node. Check to ensure it's a directory beforehand.
	pub fn children(&self) -> &Vec<Entry> {
		match &self.kind {
			EntryKind::Directory { children } => children,
			_ => {
				// Should pre-verify first, and thus never reach this point
				panic!("Attempted to retrieve children of a VFS leaf node.");
			}
		}
	}

	/// Panics if used on a leaf node. Check to ensure it's a directory beforehand.
	pub fn children_mut(&mut self) -> &mut Vec<Entry> {
		match &mut self.kind {
			EntryKind::Directory { children } => children,
			_ => {
				// Should pre-verify first, and thus never reach this point
				panic!("Attempted to mutably retrieve children of a VFS leaf node.");
			}
		}
	}

	fn cmp_name(a: &Entry, b: &Entry) -> std::cmp::Ordering {
		if a.is_leaf() && b.is_dir() {
			std::cmp::Ordering::Greater
		} else if a.is_dir() && b.is_leaf() {
			std::cmp::Ordering::Less
		} else {
			a.name.partial_cmp(&b.name).unwrap()
		}
	}

	fn sort(&mut self) {
		match &mut self.kind {
			EntryKind::Leaf { .. } => {}
			EntryKind::Directory { children } => {
				children.sort_by(Self::cmp_name);

				for child in children {
					child.sort();
				}
			}
		}
	}

	fn lookup<'s>(&self, mut iter: impl Iterator<Item = &'s str>) -> Option<&Entry> {
		let p = match iter.next() {
			Some(p) => p,
			None => {
				return Some(self);
			}
		};

		let children = self.children();

		for e in children {
			if p != e.name {
				continue;
			}

			return Self::lookup(e, iter);
		}

		None
	}

	/// Case-insensitive counterpart to [`Entry::lookup`].
	fn lookup_nocase<'s>(&self, mut iter: impl Iterator<Item = &'s str>) -> Option<&Entry> {
		let p = match iter.next() {
			Some(p) => p,
			None => {
				return Some(self);
			}
		};

		let children = self.children();

		for e in children {
			if !p.eq_ignore_ascii_case(&e.name) {
				continue;
			}

			return Self::lookup_nocase(e, iter);
		}

		None
	}
}

// Internal implementation details: anything related to mounting.
impl VirtualFs {
	/// Forwards files of an as-yet unknown kind to the right mounting function.
	fn mount_file(bytes: Vec<u8>, mount_name: &str) -> Result<Entry, Error> {
		match is_valid_wad(&bytes[..], bytes.len().try_into().unwrap()) {
			Ok(b) => {
				if b {
					// If this WAD was nested in another archive,
					// it will need to have its extension taken off
					return Self::mount_wad(bytes, mount_name.split('.').next().unwrap());
				}
			}
			Err(err) => {
				warn!(
					"Failed to determine if file is a WAD: {}
					Error: {}",
					mount_name, err
				);
				return Err(Error::IoError(err));
			}
		};

		if is_zip(&bytes) {
			// If this zip file was nested in another archive,
			// it will need to have its extension taken off
			return Self::mount_zip(bytes, mount_name.split('.').next().unwrap());
		}

		// This isn't any kind of archive. Mount whatever it may be

		Ok(Entry {
			name: mount_name.to_owned(),
			kind: EntryKind::Leaf { bytes },
		})
	}

	fn mount_zip(bytes: Vec<u8>, mount_name: &str) -> Result<Entry, Error> {
		let cursor = Cursor::new(&bytes);
		let mut zip = ZipArchive::new(cursor).map_err(Error::ZipError)?;

		let mut ret = Entry {
			name: mount_name.to_string(),
			kind: EntryKind::Directory {
				children: Vec::<Entry>::default(),
			},
		};

		for i in 0..zip.len() {
			let mut zfile = match zip.by_index(i) {
				// Zip directories get constructed when mounting
				// leaf files that rely on them
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
						mount_name, err
					);
					continue;
				}
			};

			let zfsize = zfile.size();
			let mut bytes = Vec::<u8>::with_capacity(zfsize.try_into().unwrap());

			match zfile.enclosed_name() {
				Some(_) => {}
				None => {
					warn!(
						"A zip file entry contains an unsafe path at index: {}
						Zip file mount name: {}",
						i, mount_name
					);
					continue;
				}
			}

			match io::copy(&mut zfile, &mut bytes) {
				Ok(count) => {
					if count != zfsize {
						warn!(
							"Failed to read all bytes of zip file entry: {}
							Zip file mount name: {}",
							zfile.enclosed_name().unwrap().display(),
							mount_name
						);
						continue;
					}
				}
				Err(err) => {
					warn!(
						"Failed to read zip file entry: {}
						Zip file mount name: {}
						Error: {}",
						zfile.enclosed_name().unwrap().display(),
						mount_name,
						err
					);
					continue;
				}
			};

			let zfpath = match zfile.enclosed_name() {
				Some(en) => en,
				None => {
					warn!(
						"Zip file contains unsafe path at index {}: {}",
						i, mount_name
					);
					continue;
				}
			};

			let iter = str_iter_from_path(zfpath);
			let counter = zfpath.size();
			Self::mount_zip_recur(&mut ret, iter, counter, bytes);
		}

		Ok(ret)
	}

	fn mount_zip_recur<'a>(
		parent: &mut Entry,
		mut iter: impl Iterator<Item = &'a str>,
		mut counter: usize,
		bytes: Vec<u8>,
	) {
		let comp = match iter.next() {
			Some(c) => c,
			None => {
				return;
			}
		};

		counter -= 1;

		let children = parent.children_mut();

		if counter == 0 {
			// Time to push a leaf node. This could be a zip, a WAD, or neither
			match Self::mount_file(bytes, comp) {
				Ok(entry) => {
					children.push(entry);
					return;
				}
				Err(err) => {
					warn!(
						"Failed to mount zip file: {}\nError: {}",
						iter.collect::<PathBuf>().join(comp).display(),
						err
					);
					return;
				}
			};
		}

		// Not at the path's end yet. A directory may exist at this path component;
		// if so, push the new entry on to it. Otherwise, create that new dir.,
		// and then recur into it

		let mut recur_into = children.len();

		for (i, sub) in children.iter().enumerate() {
			if sub.name != comp {
				continue;
			}

			recur_into = i;
			break;
		}

		if recur_into != children.len() {
			Self::mount_zip_recur(children.get_mut(recur_into).unwrap(), iter, counter, bytes);
		} else {
			children.push(Entry {
				name: comp.to_owned(),
				kind: EntryKind::Directory {
					children: Vec::<Entry>::default(),
				},
			});

			Self::mount_zip_recur(children.last_mut().unwrap(), iter, counter, bytes);
		}
	}

	fn mount_wad(bytes: Vec<u8>, mount_name: &str) -> Result<Entry, Error> {
		lazy_static! {
			static ref RGX_NOMOUNT: Regex = Regex::new(r"[SPF][12]*_(?:START|END)")
				.expect("Failed to evaluate `VirtualFs::mount_wad::RGX_NOMOUNT`.");
			static ref RGXSET_MAPMARKER: RegexSet =
				RegexSet::new(&[r"MAP[0-9]{2}", r"E[0-9]M[0-9]", r"HUBMAP"])
					.expect("Failed to evaluate `VirtualFs::mount_wad::RGXSET_MAPMARKER`.");
			static ref RGXSET_MAPPART: RegexSet = RegexSet::new(&[
				r"THINGS",
				r"LINEDEFS",
				r"SIDEDEFS",
				r"VERTEXES",
				r"SEGS",
				r"SSECTORS",
				r"NODES",
				r"SECTORS",
				r"REJECT",
				r"BLOCKMAP",
				r"BEHAVIOR",
				// UDMF
				r"TEXTMAP",
				r"DIALOGUE",
				r"ZNODES",
				r"SCRIPTS",
				// Note: ENDMAP gets filtered out, since there's no need to keep it
			])
			.expect("Failed to evaluate `VirtualFs::mount_wad::RGXSET_MAPPART`.");
		};

		let wad = wad::parse_wad(bytes).map_err(Error::WadError)?;
		let mut dissolution = wad.dissolve();

		let mut children = Vec::<Entry>::default();
		let mut mapfold: Option<Entry> = None;

		for (ebytes, name) in dissolution.drain(..) {
			// No need to keep markers delimiting graphics sections
			if RGX_NOMOUNT.is_match(&name) {
				continue;
			}

			if RGXSET_MAPMARKER.is_match(&name) {
				match mapfold.take() {
					Some(entry) => {
						children.push(entry);
					}
					None => {}
				};

				mapfold = Some(Entry {
					name,
					kind: EntryKind::Directory {
						children: Default::default(),
					},
				});
				continue;
			}

			let dup_pos = children.iter().position(|entry| entry.name == name);

			match dup_pos {
				None => {}
				Some(pos) => {
					let mut entry = children.swap_remove(pos);

					match entry.kind {
						EntryKind::Leaf { .. } => {
							let mut sub_children = Vec::<Entry>::default();
							entry.name = "000".to_string();

							sub_children.push(entry);
							sub_children.push(Entry {
								name: "001".to_string(),
								kind: EntryKind::Leaf { bytes: ebytes },
							});

							let new_folder = Entry {
								name,
								kind: EntryKind::Directory {
									children: sub_children,
								},
							};

							children.push(new_folder);
						}
						EntryKind::Directory { mut children } => {
							children.push(Entry {
								name: format!("{:03}", children.len()),
								kind: EntryKind::Leaf { bytes: ebytes },
							});
						}
					}

					continue;
				}
			}

			let pop_map = match &mut mapfold {
				Some(entry) => {
					if RGXSET_MAPPART.is_match(&name) {
						entry.children_mut().push(Entry {
							name,
							kind: EntryKind::Leaf { bytes: ebytes },
						});
						continue;
					} else {
						true
					}
				}
				None => {
					children.push(Entry {
						name,
						kind: EntryKind::Leaf { bytes: ebytes },
					});
					continue;
				}
			};

			if pop_map {
				children.push(mapfold.take().unwrap());
			}
		}

		if mapfold.is_some() {
			children.push(mapfold.take().unwrap());
		}

		Ok(Entry {
			name: mount_name.to_string(),
			kind: EntryKind::Directory { children },
		})
	}

	fn mount_dir(real_path: &Path, mount_name: &str) -> Result<Entry, Error> {
		let mut children = Vec::<Entry>::default();

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

			let res = if ftype.is_dir() {
				Self::mount_dir(&entry_path, fname)
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

				Self::mount_file(bytes, fname)
			};

			match res {
				Ok(e) => {
					children.push(e);
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

		Ok(Entry {
			name: mount_name.to_owned(),
			kind: EntryKind::Directory { children },
		})
	}
}

impl Default for VirtualFs {
	fn default() -> Self {
		VirtualFs {
			root: Entry {
				name: String::from("/"),
				kind: EntryKind::Directory {
					children: Default::default(),
				},
			},
		}
	}
}

lazy_static! {
	static ref RGX_INVALIDMOUNTPATH: Regex = Regex::new(r"[^A-Za-z0-9-_/\.]")
		.expect("Failed to evaluate `VirtualFs::RGX_INVALIDMOUNTPATH`.");
}

// Traits for Impure-specific functionality ////////////////////////////////////

/// A separate trait provides functions that are specific to Impure, so that the
/// VFS itself can later be more easily made into a standalone library.
pub trait ImpureVfs {
	/// On the debug build, attempt to mount `/env::current_dir()/data`.
	/// On the release build, attempt to mount `/utils::exe_dir()/impure.zip`.
	fn mount_enginedata(&mut self) -> Result<(), Error>;
	fn mount_gamedata(&mut self, paths: &[PathBuf]) -> Vec<GameDataMeta>;

	/// See [`ImpureVfsEntry::is_impure_package`].
	/// Returns `None` if and only if nothing exists at the given path.
	fn is_impure_package(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`ImpureVfsEntry::is_udmf_map`].
	/// Returns `None` if and only if nothing exists at the given path.
	fn is_udmf_map(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`ImpureVfsEntry::has_zscript`].
	/// Returns `None` if and only if nothing exists at the given path.
	fn has_zscript(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`ImpureVfsEntry::has_edfroot`].
	/// Returns `None` if and only if nothing exists at the given path.
	fn has_edfroot(&self, path: impl AsRef<Path>) -> Option<bool>;

	/// See [`ImpureVfsEntry::has_decorate`].
	/// Returns `None` if and only if nothing exists at the given path.
	fn has_decorate(&self, path: impl AsRef<Path>) -> Option<bool>;

	fn parse_gamedata_meta(
		&self,
		path: impl AsRef<Path>,
	) -> Result<GameDataMeta, Box<dyn std::error::Error>>;

	fn window_icon_from_file(&self, path: impl AsRef<Path>) -> Option<winit::window::Icon>;
}

impl ImpureVfs for VirtualFs {
	fn mount_enginedata(&mut self) -> Result<(), Error> {
		#[cfg(not(debug_assertions))]
		{
			let path: PathBuf = [exe_dir(), PathBuf::from("impure.zip")].iter().collect();
			self.mount(&[(path, "/impure")]).pop().unwrap()
		}
		#[cfg(debug_assertions)]
		{
			use std::env;

			let path: PathBuf = [
				env::current_dir().map_err(Error::IoError)?,
				PathBuf::from("data"),
			]
			.iter()
			.collect();

			self.mount(&[(path, "/impure")]).pop().unwrap()
		}
	}

	fn mount_gamedata(&mut self, paths: &[PathBuf]) -> Vec<GameDataMeta> {
		let call_time = std::time::Instant::now();
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
						"Skipping unsupported gamedata entry: {}",
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
			let is_impure_package = self.is_impure_package(&to_mount[i].1).expect(
				"Failed to lookup a newly-mounted item."
			);

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
				GameDataMeta::new(uuid, vers)
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
		self.lookup(path).map(|entry| entry.is_impure_package())
	}

	fn is_udmf_map(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|entry| entry.is_udmf_map())
	}

	fn has_zscript(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|entry| entry.has_zscript())
	}

	fn has_edfroot(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|entry| entry.has_edfroot())
	}

	fn has_decorate(&self, path: impl AsRef<Path>) -> Option<bool> {
		self.lookup(path).map(|entry| entry.has_decorate())
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
}

/// A separate trait provides functions that are specific to Impure, so that the
/// VFS itself can later be more easily made into a standalone library.
pub trait ImpureVfsEntry {
	/// Check if a directory node has a `meta.toml` leaf (case-insensitive) in it.
	/// Unconditionally returns false if this entry is, itself, a leaf node.
	fn is_impure_package(&self) -> bool;
	/// Check if this is a directory with a leaf node named `TEXTMAP`.
	/// Unconditionally returns false if this entry is, itself, a leaf node.
	fn is_udmf_map(&self) -> bool;
	/// Check if a directory node has a `decorate` file (case-insensitive) in it.
	/// Unconditionally returns false if this entry is, itself, a leaf node.
	fn has_decorate(&self) -> bool;
	/// Check if a directory node has a `zscript` file (case-insensitive) in it.
	/// Unconditionally returns false if this entry is, itself, a leaf node.
	fn has_zscript(&self) -> bool;
	/// Check if a directory node has an `edfroot` file (case-insensitive) in it.
	/// Unconditionally returns false if this entry is, itself, a leaf node.
	fn has_edfroot(&self) -> bool;
}

impl ImpureVfsEntry for Entry {
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

struct ZsProxyFs<'v> {
	vfs: &'v VirtualFs,
	root: &'v str
}

impl<'v> ZsFileSystem for ZsProxyFs<'v> {
	fn get_file(&mut self, filename: &str) -> Option<ZsFile> {
		let rel_root = self.vfs.lookup(self.root).expect(
			"`ZsProxyFs::get_file` failed to find its relative root."
		);

		let path = Path::new(filename);
		let iter = str_iter_from_path(path);
		let target = match rel_root.lookup_nocase(iter) {
			Some(e) => e,
			None => {
				warn!("Failed to find ZScript file: {}", filename);
				return None;
			}
		};

		match &target.kind {
			EntryKind::Directory { .. } => {
				warn!("Expected ZScript file, found directory: {}", filename);
				None
			}
			EntryKind::Leaf { bytes } => {
				Some(ZsFile::new(
					filename.to_string(),
					bytes.clone()
				))
			}
		}
	}

	fn get_files_no_ext(&mut self, filename: &str) -> Vec<ZsFile> {
		let mut ret = Vec::<ZsFile>::default();

		let rel_root = self.vfs.lookup(self.root).expect(
			"`ZsProxyFs::get_files_no_ext` failed to find its relative root."
		);

		for child in rel_root.children() {
			let mut noext = child.name.splitn(2, '.');

			let bytes = if let EntryKind::Leaf { bytes } = &child.kind {
				bytes
			} else {
				continue;
			};

			let stem = match noext.next() {
				Some(s) => s,
				None => { continue; }
			};

			if stem.eq_ignore_ascii_case(filename) {
				ret.push(ZsFile::new(
					filename.to_string(),
					bytes.clone()
				));
			}
		}

		ret
	}
}

// Error type //////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
	/// A path argument failed to canonicalize somehow.
	Canonicalization(io::Error),
	/// Failure to read the entries of a directory the caller wanted to mount.
	DirectoryRead(io::Error),
	/// The caller provided a mount point that isn't comprised solely of
	/// alphanumeric characters, underscores, dashes, periods, and forward slashes.
	InvalidMountPoint,
	/// A path argument did not pass a UTF-8 validity check.
	InvalidUtf8,
	IoError(io::Error),
	/// Trying to mount something onto `DOOM2/PLAYPAL`, for example, is illegal.
	MountToLeaf,
	/// The caller attempted to lookup/read/write/unmount a non-existent file.
	NonExistentEntry,
	/// The caller attempted to lookup/read/write/mount a non-existent file.
	NonExistentFile,
	/// The caller attempted to read a directory, archive, or WAD.
	Unreadable,
	/// The caller attempted to mount something to a point which
	/// already had something mounted onto it.
	Remount,
	/// The caller attempted to illegally mount a symbolic link.
	SymlinkMount,
	WadError(wad::Error),
	ZipError(ZipError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Canonicalization(err) => {
				write!(f, "Failed to canonicalize given path: {}", err)
			}
			Self::DirectoryRead(err) => {
				write!(f, "Failed to read a directory: {}", err)
			}
			Self::InvalidMountPoint => {
				write!(
					f,
					"Mount point can only contain letters, numbers, underscores, \
					periods, dashes, and forward slashes."
				)
			}
			Self::InvalidUtf8 => {
				write!(f, "Given path failed to pass a UTF-8 validity check.")
			}
			Self::IoError(err) => {
				write!(f, "{}", err)
			}
			Self::MountToLeaf => {
				write!(
					f,
					"Attempted to mount something using an existing leaf node \
					as part of the mount point."
				)
			}
			Self::NonExistentEntry => {
				write!(f, "Attempted to operate on a non-existent entry.")
			}
			Self::NonExistentFile => {
				write!(f, "Attempted to operate on a non-existent file.")
			}
			Self::Unreadable => {
				write!(
					f,
					"Directories, archives, and WADs are ineligible for reading."
				)
			}
			Self::Remount => {
				write!(f, "Something is already mounted to the given path.")
			}
			Self::SymlinkMount => {
				write!(f, "Symbolic links can not be mounted.")
			}
			Self::WadError(err) => {
				write!(f, "{}", err)
			}
			Self::ZipError(err) => {
				write!(f, "{}", err)
			}
		}
	}
}
