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

use crate::{
	data::{GamedataKind, GamedataMeta},
	utils::*,
};
use lazy_static::lazy_static;
use log::{error, info, warn};
use regex::{Regex, RegexSet};
use std::{
	fmt,
	fs::{self, File},
	io::{self, Read},
	path::{Path, PathBuf},
};
use zip::ZipArchive;

#[derive(Debug)]
pub enum Error {
	/// A path argument failed to canonicalize somehow.
	Canonicalization(io::Error),
	/// The caller provided a mount point that isn't comprised solely of
	/// alphanumeric characters, underscores, dashes, periods, and forward slashes.
	InvalidMountPoint,
	/// A path argument did not pass a UTF-8 validity check.
	InvalidUtf8,
	IoError(io::Error),
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
	Other(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Canonicalization(err) => {
				write!(f, "Failed to canonicalize given path: {}", err)
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
			Self::Other(s) => {
				write!(f, "{}", s)
			}
		}
	}
}

struct Entry {
	name: String,
	kind: EntryKind,
}

enum EntryKind {
	/// Any loose file that isn't otherwise some kind of archive.
	/// For instance, a DEHACKED lump, or a custom lump for a mod.
	File {
		real_path: PathBuf,
	},
	Directory {
		sub_entries: Vec<Entry>,
	},
	/// Used to implement the directory tree root,
	/// as well as the directories in archives.
	FakeDirectory {
		sub_entries: Vec<Entry>,
	},
	Wad {
		wad: wad::Wad,
		sub_entries: Vec<Entry>,
	},
	WadEntry {
		id: wad::EntryId,
	},
	Zip {
		real_path: PathBuf,
		sub_entries: Vec<Entry>,
	},
	ZipEntry {
		index: usize,
	},
}

impl Entry {
	fn is_leaf(&self) -> bool {
		matches!(
			self.kind,
			EntryKind::File { .. } | EntryKind::WadEntry { .. } | EntryKind::ZipEntry { .. }
		)
	}

	fn is_readable(&self) -> bool {
		matches!(
			self.kind,
			EntryKind::File { .. } | EntryKind::Wad { .. } | EntryKind::Zip { .. }
		)
	}

	fn sub_entries(&self) -> Option<&Vec<Entry>> {
		match &self.kind {
			EntryKind::Directory { sub_entries, .. }
			| EntryKind::Wad { sub_entries, .. }
			| EntryKind::Zip { sub_entries, .. }
			| EntryKind::FakeDirectory { sub_entries, .. } => Some(sub_entries),
			_ => None,
		}
	}

	fn sub_entries_mut(&mut self) -> Option<&mut Vec<Entry>> {
		match &mut self.kind {
			EntryKind::Directory { sub_entries, .. }
			| EntryKind::Wad { sub_entries, .. }
			| EntryKind::Zip { sub_entries, .. }
			| EntryKind::FakeDirectory { sub_entries, .. } => Some(sub_entries),
			_ => None,
		}
	}
}

pub struct VirtualFs {
	root: Entry,
}

impl Default for VirtualFs {
	fn default() -> Self {
		VirtualFs {
			root: Entry {
				name: String::from("/"),
				kind: EntryKind::FakeDirectory {
					sub_entries: Vec::<Entry>::default()
				}
			}
		}
	}
}

lazy_static! {
	static ref RGX_INVALIDMOUNTPATH: Regex = Regex::new(r"[^A-Za-z0-9-_/\.]")
		.expect("Failed to evaluate `VirtualFs::mount::RGX_INVALIDMOUNTPATH`.");
}

// Public interface.
impl VirtualFs {
	pub fn mount(
		&mut self,
		path: impl AsRef<Path>,
		mount_point: impl AsRef<Path>,
	) -> Result<(), Error> {
		let p = path.as_ref();
		let mpoint = mount_point.as_ref();

		// Ensure file to mount is even supported

		match Self::mount_supported(p) {
			Ok(()) => {}
			Err(err) => {
				return Err(err);
			}
		};

		// Ensure mount point is only alphanumerics, underscores, dashes, slashes

		{
			let mpstr = match mpoint.to_str() {
				Some(s) => s,
				None => {
					return Err(Error::InvalidMountPoint);
				}
			};

			if RGX_INVALIDMOUNTPATH.is_match(mpstr) {
				return Err(Error::InvalidMountPoint);
			}
		}

		// Ensure something exists at real path

		if !p.exists() {
			return Err(Error::NonExistentFile);
		}

		// Ensure nothing already exists at end of mount point

		if self.lookup(mpoint).is_ok() {
			return Err(Error::Remount);
		}

		// Convert real path to its canonical form

		let canon = match p.canonicalize() {
			Ok(c) => c,
			Err(err) => {
				return Err(Error::Canonicalization(err));
			}
		};

		// Create fake directories to get to mount point if necessary

		let mp_base = mpoint.parent().ok_or(Error::InvalidMountPoint)?;

		if !mp_base.is_empty() {
			let iter = mp_base.ancestors().fuse();

			for ppath in iter {
				if self.lookup(ppath).is_err() {
					Self::build_mount_point(&mut self.root, str_iter_from_path(mp_base))?;
					break;
				}
			}
		}

		let mpe = self
			.lookup_mut(mp_base)
			.expect("`VirtualFs::build_mount_point()` failed.");

		let mount_name = mpoint
			.file_name()
			.ok_or(Error::NonExistentEntry)?
			.to_str()
			.ok_or(Error::InvalidUtf8)?;

		let res = if canon.is_dir() {
			Self::mount_dir(&canon, mount_name, mpe)
		} else {
			Self::mount_file(&canon, mount_name, mpe)
		};

		if res.is_ok() {
			info!(
				"VFS mounted: \"{}\" as \"{}\".",
				canon.display(),
				mpoint.display()
			);
		}

		res
	}

	pub fn mount_supported(path: impl AsRef<Path>) -> Result<(), Error> {
		let p = path.as_ref();

		if !p.exists() {
			return Err(Error::NonExistentFile);
		}

		if p.is_dir() {
			return Ok(());
		}

		if p.is_symlink() {
			return Err(Error::SymlinkMount);
		}

		Ok(())
	}

	pub fn exists(&self, path: impl AsRef<Path>) -> bool {
		self.lookup(path).is_ok()
	}

	pub fn is_dir(&self, path: impl AsRef<Path>) -> bool {
		match self.lookup(path) {
			Ok(entry) => !entry.is_leaf(),
			Err(_) => false,
		}
	}

	pub fn file_names(&self, path: impl AsRef<Path>) -> Vec<String> {
		let mut ret = Vec::<String>::default();

		let entry = match self.lookup(path) {
			Ok(e) => e,
			Err(_) => {
				return ret;
			}
		};

		if entry.is_leaf() {
			return ret;
		}

		let subs = entry.sub_entries().unwrap();

		for sub in subs {
			ret.push(sub.name.clone());
		}

		ret
	}

	/// An error will be returned if the given path is invalid, or the requested
	/// entry is non-existent.
	pub fn read_bytes(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
		let p = path.as_ref();
		let iter = str_iter_from_path(p);
		let to_read = Self::lookup_to_read(&self.root, iter)?;

		match &to_read.kind {
			EntryKind::File { real_path, .. } => fs::read(&real_path)
				.map_err(|_| Error::Other(format!("Malformed file: {}", real_path.display()))),
			EntryKind::Wad { wad, .. } => {
				let entry = self.lookup(p)?;

				if let EntryKind::WadEntry { id } = &entry.kind {
					let idb = id.as_bytes();
					let idc = wad::EntryId::from_bytes(idb);

					match wad.by_id(idc) {
						Some(lump) => Ok(lump.to_owned()),
						None => {
							panic!("An illegal WAD entry ID was stored.");
						}
					}
				} else {
					Err(Error::Other(
						"Mismatch between readable and leaf entry.".to_string(),
					))
				}
			}
			EntryKind::Zip { real_path, .. } => {
				let file = match File::open(&real_path) {
					Ok(f) => f,
					Err(err) => {
						return Err(Error::Other(format!("{}", err)));
					}
				};

				let mut zip = match ZipArchive::new(file) {
					Ok(z) => z,
					Err(err) => {
						return Err(Error::Other(format!("{}", err)));
					}
				};

				let entry = self.lookup(p)?;

				if let EntryKind::ZipEntry { index } = &entry.kind {
					let mut zfile = match zip.by_index(*index) {
						Ok(zf) => zf,
						Err(err) => {
							return Err(Error::Other(format!("Zip file read failure: {}", err)));
						}
					};

					let mut ret = Vec::<u8>::with_capacity(zfile.size() as usize);

					match zfile.read_to_end(&mut ret) {
						Ok(bytes_read) => {
							if bytes_read != ret.capacity() {
								return Err(Error::Other(format!(
									"Incomplete file read of zip entry; 
										expected {}, got {}.",
									ret.capacity(),
									bytes_read
								)));
							}
						}
						Err(err) => return Err(Error::Other(format!("Zip read failed: {}", err))),
					};

					Ok(ret)
				} else {
					Err(Error::Other(
						"Mismatch between readable and leaf entry.".to_string(),
					))
				}
			}
			_ => Err(Error::Unreadable),
		}
	}

	pub fn read_string(&self, path: impl AsRef<Path>) -> Result<String, Error> {
		let bytes = self.read_bytes(path)?;
		Ok(String::from_utf8_lossy(&bytes[..]).to_string())
	}
}

// Internal implementation details: anything related to mounting.
impl VirtualFs {
	fn build_mount_point<'s>(
		parent: &mut Entry,
		mut iter: impl Iterator<Item = &'s str>,
	) -> Result<(), Error> {
		let p = match iter.next() {
			Some(p) => p,
			None => {
				return Ok(());
			}
		};

		let subs = parent
			.sub_entries_mut()
			.expect("`VirtualFs::build_mount_point()` expected a fake directory.");

		for e in subs.iter_mut() {
			if e.name != p {
				continue;
			}

			if e.is_leaf() {
				return Err(Error::Other(
					"Attempted to build mount point onto a leaf entry.".to_string(),
				));
			}

			// As per the above check, this should never fail
			let subs = e.sub_entries_mut().unwrap();

			subs.push(Entry {
				name: String::from(p),
				kind: EntryKind::FakeDirectory {
					sub_entries: Vec::<Entry>::default(),
				},
			});

			return Self::build_mount_point(subs.last_mut().unwrap(), iter);
		}

		// If we made it here, the given entry has no sub-entry corresponding
		// to the intended component in the mount point, and it needs to be pushed

		subs.push(Entry {
			name: String::from(p),
			kind: EntryKind::FakeDirectory {
				sub_entries: Vec::<Entry>::default(),
			},
		});

		return Self::build_mount_point(subs.last_mut().unwrap(), iter);
	}

	/// Forwards files of an as-yet unknown kind to the right mounting function.
	/// `parent` should be the pre-existing entry that the new entry will be
	/// pushed onto. Thus, if the given mount point is "/DOOM2", the given parent
	/// should be `Self::root`.
	fn mount_file(path: &Path, mount_name: &str, parent: &mut Entry) -> Result<(), Error> {
		if path.is_symlink() {
			return Err(Error::SymlinkMount);
		}

		if has_gzdoom_extension(path) || has_zip_extension(path) || has_eternity_extension(path) {
			return Self::mount_zip(path, mount_name, parent);
		}

		if has_wad_extension(path) {
			match is_valid_wad(path) {
				Ok(b) => {
					if b {
						return Self::mount_wad(path, mount_name, parent);
					} else {
						return Err(Error::Other(format!(
							"Attempted to mount malformed WAD file: {}",
							path.display()
						)));
					}
				}
				Err(err) => {
					return Err(Error::Other(format!(
						"Failed to determine if file is valid WAD: {}
						\n\tError: {}",
						path.display(),
						err
					)));
				}
			}
		}

		// Neither zip, nor WAD. Mount whatever this is

		let subs = parent
			.sub_entries_mut()
			.expect("`VirtualFs::mount_file()` illegally received a leaf entry.");

		subs.push(Entry {
			name: mount_name.to_owned(),
			kind: EntryKind::File {
				real_path: path.to_owned(),
			},
		});

		Ok(())
	}

	/// `parent` should be the pre-existing entry that the new entry will be
	/// pushed onto. Thus, if the given mount point is "/DOOM2", the given parent
	/// should be `Self::root`.
	fn mount_zip(path: &Path, mount_name: &str, parent: &mut Entry) -> Result<(), Error> {
		let file = match File::open(path) {
			Ok(f) => f,
			Err(err) => {
				return Err(Error::Other(format!("{}", err)));
			}
		};

		let mut zip = match ZipArchive::new(file) {
			Ok(z) => z,
			Err(err) => {
				return Err(Error::Other(format!("{}", err)));
			}
		};

		let new = Entry {
			name: mount_name.to_owned(),
			kind: EntryKind::Zip {
				real_path: path.to_owned(),
				sub_entries: Vec::<Entry>::default(),
			},
		};

		// Grody workaround for loop mutability
		let new = Some(new);
		let new = std::cell::RefCell::new(new);

		for i in 0..zip.len() {
			let zfile = match zip.by_index(i) {
				Ok(zf) => zf,
				Err(err) => {
					warn!(
						"Zip file contains bad index {}: {}\nError: {}",
						path.display(),
						i,
						err
					);
					continue;
				}
			};

			let fpath = match zfile.enclosed_name() {
				Some(fp) => fp,
				None => {
					warn!(
						"Zip file contains unsafe path at index {}: {}",
						i,
						path.display()
					);
					continue;
				}
			};

			let iter = str_iter_from_path(fpath).fuse();
			let counter = str_iter_from_path(fpath).fuse().count();

			let kind: EntryKind = if zfile.is_dir() {
				EntryKind::FakeDirectory {
					sub_entries: Vec::<Entry>::default(),
				}
			} else {
				EntryKind::ZipEntry { index: i }
			};

			Self::mount_zip_recur(new.borrow_mut().as_mut().unwrap(), iter, counter, kind);
		}

		let subs = parent
			.sub_entries_mut()
			.expect("`VirtualFs::mount_zip()` expected a non-leaf parent.");

		subs.push(new.take().unwrap());

		Ok(())
	}

	/// `parent` should be the pre-existing entry that the new entry will be
	/// pushed onto. Thus, if the given mount point is "/DOOM2", the given parent
	/// should be `Self::root`.
	fn mount_zip_recur<'a>(
		parent: &mut Entry,
		mut iter: impl Iterator<Item = &'a str>,
		mut counter: usize,
		kind: EntryKind,
	) {
		let comp = match iter.next() {
			Some(c) => c,
			None => {
				return;
			}
		};

		counter -= 1;

		let subs = parent
			.sub_entries_mut()
			.expect("`VirtualFs::mount_zip_recur()` expected a non-leaf parent.");

		if counter == 0 {
			subs.push(Entry {
				name: comp.to_owned(),
				kind,
			});

			return;
		}

		// Not at the path's end yet. A directory may exist at this path component;
		// if so, push the new entry on to it. Otherwise, create that new dir.,
		// and then recur into it

		let mut recur_into = subs.len();

		for (i, sub) in subs.iter().enumerate() {
			if sub.name != comp {
				continue;
			}

			recur_into = i;
			break;
		}

		if recur_into != subs.len() {
			Self::mount_zip_recur(subs.get_mut(recur_into).unwrap(), iter, counter, kind);
		} else {
			subs.push(Entry {
				name: comp.to_owned(),
				kind: EntryKind::FakeDirectory {
					sub_entries: Vec::<Entry>::default(),
				},
			});

			Self::mount_zip_recur(subs.last_mut().unwrap(), iter, counter, kind);
		}
	}

	/// `parent` should be the pre-existing entry that the new entry will be
	/// pushed onto. Thus, if the given mount point is "/DOOM2", the given parent
	/// should be `Self::root`.
	fn mount_dir(path: &Path, mount_name: &str, parent: &mut Entry) -> Result<(), Error> {
		let subs = parent
			.sub_entries_mut()
			.expect("`VirtualFs::mount_dir()` expected a non-leaf node.");

		subs.push(Entry {
			name: mount_name.to_owned(),
			kind: EntryKind::Directory {
				sub_entries: Vec::<Entry>::default(),
			},
		});

		// Now, check under this directory for other files/dirs/zips/WADs

		let new = subs.last_mut().unwrap();

		let reader = match fs::read_dir(path) {
			Ok(r) => r,
			Err(err) => {
				return Err(Error::Other(format!("{}", err)));
			}
		};

		for deres in reader {
			if deres.is_err() {
				warn!(
					"Skipping malformed directory entry under: {}",
					path.display()
				);
				continue;
			}

			let dentry = deres.unwrap();

			match dentry.file_type() {
				Ok(ft) => {
					if ft.is_symlink() {
						continue;
					}

					let fname = dentry.file_name();
					let fnamestr = fname.to_str();

					if fnamestr.is_none() {
						warn!(
							"Dir. entry with invalid UTF-8 in file name will \
							not be mounted: {}",
							dentry.path().display()
						);
						continue;
					}

					let dp = dentry.path();

					let res = if ft.is_dir() {
						Self::mount_dir(&dp, fnamestr.unwrap(), new)
					} else {
						Self::mount_file(&dp, fnamestr.unwrap(), new)
					};

					match res {
						Ok(()) => {}
						Err(err) => {
							warn!("Failed to mount: {}\nError: {}", dp.display(), err);
						}
					};
				}
				Err(err) => {
					warn!(
						"Failed to determine type of dir. entry; skipping: {:?}
						\nError: {}",
						dentry.file_name(),
						err
					);
					continue;
				}
			};
		}

		Ok(())
	}

	/// `parent` should be the pre-existing entry that the new entry will be
	/// pushed onto. Thus, if the given mount point is "/DOOM2", the given parent
	/// should be `Self::root`.
	fn mount_wad(path: &Path, mount_name: &str, parent: &mut Entry) -> Result<(), Error> {
		let wad = match wad::load_wad_file(path) {
			Ok(w) => w,
			Err(err) => {
				return Err(Error::Other(format!("{}", err)));
			}
		};

		let subs = parent
			.sub_entries_mut()
			.expect("`VirtualFs::mount_wad()` expected a non-leaf parent.");

		let mut subsubs = Vec::<Entry>::default();

		for went in wad.entry_iter() {
			subsubs.push(Entry {
				name: went.display_name().to_owned(),
				kind: EntryKind::WadEntry { id: went.id },
			})
		}

		subs.push(Entry {
			name: mount_name.to_owned(),
			kind: EntryKind::Wad {
				wad,
				sub_entries: subsubs,
			},
		});

		Ok(())
	}
}

// Internal implementation details: lookup functions.
impl VirtualFs {
	fn lookup(&self, path: impl AsRef<Path>) -> Result<&Entry, Error> {
		let p = path.as_ref();

		if p.is_empty() || p.is_root() {
			return Ok(&self.root);
		}

		let mut iter = str_iter_from_path(path.as_ref());

		let p = match iter.next() {
			Some(n) => {
				if n == "/" {
					iter.next().ok_or(Error::NonExistentEntry)?
				} else {
					n
				}
			}
			None => {
				return Err(Error::NonExistentEntry);
			}
		};

		let subs = self
			.root
			.sub_entries()
			.expect("Lookup miss on a VFS leaf entry.");

		for entry in subs {
			if p != entry.name {
				continue;
			}

			return Self::lookup_recur(entry, iter);
		}

		Err(Error::NonExistentEntry)
	}

	fn lookup_recur<'a>(
		parent: &Entry,
		mut iter: impl Iterator<Item = &'a str>,
	) -> Result<&Entry, Error> {
		let p = match iter.next() {
			Some(p) => p,
			None => {
				return Ok(parent);
			}
		};

		let subs = parent
			.sub_entries()
			.expect("Lookup miss on a VFS leaf entry.");

		for e in subs {
			if p != e.name {
				continue;
			}

			return Self::lookup_recur(e, iter);
		}

		Err(Error::NonExistentEntry)
	}

	fn lookup_mut<'a>(&'a mut self, path: &'a Path) -> Result<&mut Entry, Error> {
		if path.is_empty() || path.is_root() {
			return Ok(&mut self.root);
		}

		let mut iter = str_iter_from_path(path);

		let p = match iter.next() {
			Some(n) => {
				if n == "/" {
					iter.next().ok_or(Error::NonExistentEntry)?
				} else {
					n
				}
			}
			None => {
				return Err(Error::NonExistentEntry);
			}
		};

		let subs = self
			.root
			.sub_entries_mut()
			.expect("Lookup miss on a VFS leaf entry.");

		for entry in subs {
			if p != entry.name {
				continue;
			}

			return Self::lookup_recur_mut(entry, iter);
		}

		Err(Error::NonExistentEntry)
	}

	fn lookup_recur_mut<'a>(
		parent: &'a mut Entry,
		mut iter: impl Iterator<Item = &'a str>,
	) -> Result<&'a mut Entry, Error> {
		let p = match iter.next() {
			Some(p) => p,
			None => {
				return Ok(parent);
			}
		};

		let subs = parent
			.sub_entries_mut()
			.expect("Lookup miss on a VFS leaf entry.");

		for e in subs {
			if p != e.name {
				continue;
			}

			return Self::lookup_recur_mut(e, iter);
		}

		Err(Error::NonExistentEntry)
	}

	/// If attempting to read a file in an archive or a WAD entry, one needs
	/// the full chain of entries in addition to the entry with the real path to
	/// the archive or WAD itself. This retrieves the latter.
	fn lookup_to_read<'a>(
		parent: &'a Entry,
		mut iter: impl Iterator<Item = &'a str>,
	) -> Result<&'a Entry, Error> {
		let p = match iter.next() {
			Some(n) => {
				if n == "/" {
					iter.next().ok_or(Error::NonExistentEntry)?
				} else {
					n
				}
			}
			None => {
				"" // Next check may pass, so maybe nothing wrong here
			}
		};

		if parent.is_readable() {
			return Ok(parent);
		}

		let subs = match parent.sub_entries() {
			Some(s) => s,
			None => {
				return Err(Error::NonExistentEntry);
			}
		};

		for sub in subs {
			if sub.name != p {
				continue;
			}

			return Self::lookup_to_read(sub, iter);
		}

		Err(Error::Unreadable)
	}
}

/// A separate trait provides functions that are specific to Impure, so that the
/// VFS itself can later be more easily made into a standalone library.
pub trait ImpureVfs {
	fn gamedata_kind(&self, path: impl AsRef<Path>) -> Result<GamedataKind, Error>;
	fn window_icon_from_file(&self, path: impl AsRef<Path>) -> Option<winit::window::Icon>;

	fn mount_gamedata(&mut self, path: impl AsRef<Path>) -> Vec<GamedataMeta>;
	fn mount_userdata(&mut self) -> io::Result<()>;
	fn parse_gamedata_meta(
		&self,
		path: impl AsRef<Path>,
	) -> Result<GamedataMeta, Box<dyn std::error::Error>>;
}

lazy_static! {
	static ref RGXSET_GZD: RegexSet = RegexSet::new(&[
		r"^(?i)decorate",
		r"^(?i)zscript",
		r"^(?i)cvarinfo",
		r"^(?i)menudef",
		r"^(?i)sbarinfo",
		r"^(?i)zmapinfo"
	])
	.expect("Failed to evaluate `vfs::RGXSET_GZD`.");
	static ref RGXSET_ETERN: RegexSet =
		RegexSet::new(&[r"^(?i)edfroot", r"^(?i)emapinfo", r"(?i)\.edf$"])
			.expect("Failed to evaluate `vfs::RGXSET_ETERN`.");
}

impl ImpureVfs for VirtualFs {
	fn gamedata_kind(&self, path: impl AsRef<Path>) -> Result<GamedataKind, Error> {
		let entry = self.lookup(path)?;

		let check_subentries = |sub_entries: &Vec<Entry>| {
			for sub in sub_entries {
				if sub.name == "meta.toml" {
					return Some(GamedataKind::Impure);
				}

				if RGXSET_GZD.is_match(&sub.name) {
					return Some(GamedataKind::GzDoom);
				}

				if RGXSET_ETERN.is_match(&sub.name) {
					return Some(GamedataKind::Eternity);
				}
			}

			None
		};

		let check_path = |path: &Path| {
			if has_gzdoom_extension(path) {
				return Some(GamedataKind::GzDoom);
			}

			if has_eternity_extension(path) {
				return Some(GamedataKind::Eternity);
			}

			None
		};

		match &entry.kind {
			EntryKind::Wad { .. } => {
				return Ok(GamedataKind::Wad);
			}
			EntryKind::File { real_path, .. } => {
				if is_binary(real_path).map_err(Error::IoError)? {
					return Ok(GamedataKind::Text);
				}
			}
			EntryKind::Directory { sub_entries, .. } => {
				match check_subentries(sub_entries) {
					Some(kind) => return Ok(kind),
					None => {}
				};
			}
			EntryKind::Zip {
				sub_entries,
				real_path,
				..
			} => {
				match check_subentries(sub_entries) {
					Some(kind) => return Ok(kind),
					None => {}
				};

				match check_path(real_path) {
					Some(kind) => return Ok(kind),
					None => {}
				};
			}
			_ => {
				// No other kind of entry from the types above should
				// get mounted onto root, under any circumstances
				unreachable!();
			}
		};

		Ok(GamedataKind::None)
	}

	fn window_icon_from_file(&self, path: impl AsRef<Path>) -> Option<winit::window::Icon> {
		let bytes = match self.read_bytes(path) {
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

	fn mount_gamedata(&mut self, path: impl AsRef<Path>) -> Vec<GamedataMeta> {
		let call_time = std::time::Instant::now();
		let mut mounted_count = 0;
		let mut ret = Vec::<GamedataMeta>::default();

		let entries = match fs::read_dir(path) {
			Ok(entries) => entries,
			// On the dev build, this is a valid state for the dir. structure to
			// be in. If the requisite gamedata isn't found in the PWD instead,
			// *that* represents an engine failure state
			Err(err) => {
				if err.kind() != io::ErrorKind::NotFound {
					error!("Failed to read gamedata directory: {}", err);
				}

				return ret;
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
						"Failed to retrieve metadata for gamedata directory entry: {:?}
						Error: {}",
						entry.file_name(),
						err
					);
					continue;
				}
			};

			let real_path = entry.path();

			let mount_point =
				if real_path.is_dir() || is_supported_archive(&real_path).unwrap_or_default() {
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

					fstem.unwrap().to_string()
				} else if !is_binary(&real_path).unwrap_or(true) {
					let fname = entry.file_name();
					let fname = fname.into_string();

					if fname.is_err() {
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

			let vers_str = version_from_filestem(&mut mount_point);

			match self.mount(&real_path, &mount_point) {
				Ok(()) => {
					mounted_count += 1;

					let gdk = match self.gamedata_kind(&mount_point) {
						Ok(k) => k,
						Err(err) => {
							error!(
								"Failed to determine gamedata type of: {}
								Error: {}",
								mount_point, err
							);
							continue;
						}
					};

					let meta = if gdk == GamedataKind::Impure {
						let metapath: PathBuf =
							[PathBuf::from(&mount_point), PathBuf::from("meta.toml")]
								.iter()
								.collect();

						match self.parse_gamedata_meta(&metapath) {
							Ok(m) => m,
							Err(err) => {
								error!(
									"Failed to parse gamedata meta file for package: {}
									Error: {}",
									real_path.display(),
									err
								);
								continue;
							}
						}
					} else {
						let mut m = GamedataMeta::from_uuid(mount_point, gdk);
						m.version = vers_str.unwrap_or_default();
						m
					};

					ret.push(meta);
				}
				Err(err) => {
					warn!(
						"Failed to mount gamedata entry to virtual file system: {:?}
						Error: {}",
						real_path.as_path(),
						err
					);
				}
			};
		}

		info!(
			"Mounted {} gamedata objects in {} ms.",
			mounted_count,
			call_time.elapsed().as_millis()
		);
		ret
	}

	fn mount_userdata(&mut self) -> io::Result<()> {
		let user_path = match get_user_dir() {
			Some(up) => up,
			None => {
				error!(
					"Failed to retrieve userdata path. \
					Home directory path is malformed, \
					or this platform is currently unsupported."
				);
				return Err(io::ErrorKind::Other.into());
			}
		};

		let create_dir = |path: &Path| {
			let mut rm_existing = false;
			let mut mkdir = false;

			match fs::metadata(path) {
				Ok(m) => {
					if !m.is_dir() {
						rm_existing = true;
						mkdir = true;
					}
				}
				Err(err) => {
					if err.kind() == std::io::ErrorKind::NotFound {
						mkdir = true;
					} else {
						return Err(err);
					}
				}
			};

			if rm_existing {
				match fs::remove_file(path) {
					Ok(()) => {}
					Err(err) => {
						return Err(err);
					}
				};
			}

			if mkdir {
				match fs::create_dir(path) {
					Ok(()) => {}
					Err(err) => {
						return Err(err);
					}
				}
			}

			Ok(())
		};

		create_dir(&user_path)?;
		create_dir(&user_path.join("saves"))?;

		let udata_path = user_path.join("userdata");
		create_dir(&udata_path)?;

		let udata_pstr = match udata_path.to_str() {
			Some(s) => s,
			None => {
				error!("Failed to convert userdata path into a valid string.");
				return Err(io::ErrorKind::Other.into());
			}
		};

		match self.mount(udata_pstr, "/userdata") {
			Ok(()) => {}
			Err(err) => {
				return Err(io::Error::new(io::ErrorKind::Other, err));
			}
		};

		Ok(())
	}

	fn parse_gamedata_meta(
		&self,
		path: impl AsRef<Path>,
	) -> Result<GamedataMeta, Box<dyn std::error::Error>> {
		let text = self.read_string(path.as_ref())?;
		let ret: GamedataMeta = toml::from_str(&text)?;
		Ok(ret)
	}
}
