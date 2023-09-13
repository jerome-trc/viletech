#[cfg(test)]
mod test;

use std::{path::Path, sync::Arc};

use arc_swap::{ArcSwapAny, Guard};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use triomphe::ThinArc;
use util::SmallString;

#[derive(Debug)]
pub struct VirtualFs {
	root: Arc<Folder>,
	// Lookup information is kept in these maps and is not necessarily populated.
	// The editor needs none of this; it is only computed when the client loads a game.
	indices: Vec<FilePtr>,
	/// Note that keys are file prefixes (part before first `.`), not full names.
	last: FxHashMap<SmallString, FilePtr>,
	maps_valid: bool,
}

impl VirtualFs {
	#[must_use]
	pub fn root(&self) -> Ref {
		Ref::Folder {
			vfs: self,
			folder: &self.root,
		}
	}

	#[must_use]
	pub fn get(&self, path: &str) -> Option<Ref> {
		let ppath = Path::new(path);
		let mut components = ppath.components();

		if ppath.is_absolute() {
			let _ = components.next().unwrap();
		}

		let mut stack = SmallVec::<[&Arc<Folder>; 8]>::default();
		stack.push(&self.root);

		for comp in components {
			let comp_str = match comp {
				std::path::Component::Normal(osstr) => osstr.to_str().unwrap(),
				std::path::Component::CurDir => continue,
				std::path::Component::ParentDir => {
					if stack.pop().is_none() {
						return Some(self.root());
					}

					continue;
				}
				std::path::Component::Prefix(_) => todo!("handle this?"),
				std::path::Component::RootDir => unreachable!(), // Already handled.
			};

			let fold = *stack.last().unwrap();

			let (subfold, subfile) = if fold.subfolders.is_empty() {
				// Likely a WAD.
				(
					None,
					fold.files.par_iter().find_any(|vf| vf.name == comp_str),
				)
			} else if fold.files.is_empty() {
				(
					fold.subfolders
						.par_iter()
						.find_any(|vf| vf.name == comp_str),
					None,
				)
			} else {
				rayon::join(
					|| fold.subfolders.iter().find(|s| s.name == comp_str),
					|| fold.files.iter().find(|s| s.name == comp_str),
				)
			};

			if let Some(vf) = subfile {
				return Some(Ref::File {
					vfs: self,
					folder: fold,
					file: vf,
					content: vf.content.as_ref().map(|arcswap| arcswap.load()),
				});
			} else if let Some(fold) = subfold {
				stack.push(fold);
			} else {
				return None;
			}
		}

		Some(Ref::Folder {
			vfs: self,
			folder: stack.pop().unwrap(),
		})
	}

	pub fn update_lookup_info(&mut self) {
		fn indices_recur(indices: &mut Vec<FilePtr>, folder: &Arc<Folder>) {
			for subfolder in &folder.subfolders {
				indices_recur(indices, subfolder);
			}

			for (i, _) in folder.files.iter().enumerate() {
				indices.push(FilePtr {
					folder: folder.clone(),
					index: i as u32,
				});
			}
		}

		fn last_recur(last: &mut FxHashMap<SmallString, FilePtr>, folder: &Arc<Folder>) {
			for (i, file) in folder.files.iter().enumerate() {
				let short_name = file.name.split('.').next().unwrap();

				let _ = last.insert(
					short_name.into(),
					FilePtr {
						folder: folder.clone(),
						index: i as u32,
					},
				);
			}

			for subfolder in &folder.subfolders {
				last_recur(last, subfolder);
			}
		}

		let indices = std::mem::replace(&mut self.indices, vec![]);
		let last = std::mem::replace(&mut self.last, FxHashMap::default());
		let root = &self.root;

		let (indices, last) = rayon::join(
			move || {
				let mut indices = indices;
				indices_recur(&mut indices, root);
				indices
			},
			move || {
				let mut last = last;
				last_recur(&mut last, root);
				last
			},
		);

		self.indices = indices;
		self.last = last;
		self.maps_valid = true;
	}

	pub fn flush_lookup_info(&mut self) {
		self.indices.clear();
		self.last.clear();
		self.maps_valid = false;
	}

	/// Leaves only the root.
	/// Also calls [`Self::flush_lookup_info`].
	pub fn clear(&mut self) {
		self.flush_lookup_info();
		let root = Arc::get_mut(&mut self.root).unwrap();
		root.subfolders.clear();
	}
}

impl Default for VirtualFs {
	fn default() -> Self {
		Self {
			root: Arc::new(Folder {
				name: "/".into(),
				subfolders: vec![],
				files: vec![],
			}),
			indices: vec![],
			last: FxHashMap::default(),
			maps_valid: false,
		}
	}
}

#[derive(Debug)]
pub struct Folder {
	/// Always in lowercase.
	pub name: SmallString,
	/// Note to developers; if the VFS has not populated its lookup info,
	/// all of these `Arc`s will have a strong count of 1 and a weak count of 0.
	pub subfolders: Vec<Arc<Folder>>,
	pub files: Vec<VFile>,
}

#[derive(Debug)]
struct FilePtr {
	folder: Arc<Folder>,
	index: u32,
}

#[derive(Debug)]
pub struct VFile {
	/// Always in lowercase.
	pub name: SmallString,
	/// The header byte is a bit flag field. See this type's associated constants.
	pub content: Option<ArcSwapAny<ThinArc<u8, u8>>>,
}

impl VFile {
	const CONTENT_BINARY: u8 = 1 << 0;
	const CONTENT_CONSUMED: u8 = 1 << 1;
}

/// The primary interface for quick introspection into the virtual file system.
#[derive(Debug)]
pub enum Ref<'v> {
	File {
		vfs: &'v VirtualFs,
		folder: &'v Arc<Folder>,
		file: &'v VFile,
		content: Option<Guard<ThinArc<u8, u8>>>,
	},
	Folder {
		vfs: &'v VirtualFs,
		folder: &'v Arc<Folder>,
	},
}

impl Ref<'_> {
	#[must_use]
	pub fn vfs(&self) -> &VirtualFs {
		match self {
			Self::File { vfs, .. } => vfs,
			Self::Folder { vfs, .. } => vfs,
		}
	}

	/// Note that this is not a whole path,
	/// but it will include an extension, if any is present.
	#[must_use]
	pub fn name(&self) -> &str {
		match self {
			Self::File { file, .. } => file.name.as_str(),
			Self::Folder { folder, .. } => folder.name.as_str(),
		}
	}

	#[must_use]
	pub fn read_str(&self) -> &str {
		let Self::File { content, .. } = self else {
			panic!("tried to `read_str` from a virtual folder")
		};

		let Some(content) = content else {
			panic!("tried to `read_str` from an empty virtual file")
		};

		assert_eq!(
			content.header.header, 0,
			"virtual file has non-UTF-8 content or has been consumed"
		);

		// SAFETY: The absence of the binary content flag guarantees that this
		// virtual file's content was verified to be valid UTF-8 upon its creation,
		// and it has been immutable since then.
		unsafe { std::str::from_utf8_unchecked(&content.slice) }
	}

	#[must_use]
	pub fn try_consume_text(&self) -> Option<FileText> {
		let Self::File { file, .. } = self else {
			return None;
		};

		let Some(content) = &file.content else {
			return None;
		};

		let ret = content.swap(ThinArc::from_header_and_iter(
			VFile::CONTENT_CONSUMED,
			[].iter().copied(),
		));

		if ret.header.header != 0 {
			return None;
		}

		Some(FileText(ret))
	}

	/// Be aware that this will return the byte content of a UTF-8 virtual file.
	/// Consider attempting [`Self::try_consume_text`] first.
	#[must_use]
	pub fn try_consume_binary(&self) -> Option<FileBytes> {
		let Self::File { file, .. } = self else {
			return None;
		};

		let Some(content) = &file.content else {
			return None;
		};

		let ret = content.swap(ThinArc::from_header_and_iter(
			VFile::CONTENT_CONSUMED,
			[].iter().copied(),
		));

		if (ret.header.header & VFile::CONTENT_CONSUMED) != 0 {
			return None;
		}

		Some(FileBytes(ret))
	}
}

impl PartialEq for Ref<'_> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(
				Self::File {
					vfs: l_vfs,
					file: l_file,
					..
				},
				Self::File {
					vfs: r_vfs,
					file: r_file,
					..
				},
			) => std::ptr::eq(*l_vfs, *r_vfs) && std::ptr::eq(*l_file, *r_file),
			(
				Self::Folder {
					vfs: l_vfs,
					folder: l_folder,
				},
				Self::Folder {
					vfs: r_vfs,
					folder: r_folder,
				},
			) => std::ptr::eq(*l_vfs, *r_vfs) && std::ptr::eq(*l_folder, *r_folder),
			_ => false,
		}
	}
}

impl Eq for Ref<'_> {}

/// See [`Ref::try_consume_text`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileText(ThinArc<u8, u8>);

impl std::ops::Deref for FileText {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		// SAFETY: Same guarantees as reading a string from a `Ref`.
		unsafe { std::str::from_utf8_unchecked(&self.0.slice) }
	}
}

/// See [`Ref::try_consume_binary`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBytes(ThinArc<u8, u8>);

impl std::ops::Deref for FileBytes {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0.slice
	}
}
