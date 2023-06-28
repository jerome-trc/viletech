//! A decoupled wing of LithScript for serving the VileTech Engine's specific needs.

mod lith;

mod decorate;
mod zscript;

use std::{
	collections::HashMap,
	hash::Hasher,
	marker::PhantomData,
	path::{Path, PathBuf},
	sync::Arc,
};

use crate::{
	compile::{self, JitModule},
	lir, parse, Syn, Version,
};
use dashmap::DashMap;
use doomfront::{
	rowan::GreenNode,
	zdoom::{decorate::Syn as Decorate, zscript::Syn as ZScript},
	LangExt, ParseTree,
};
use parking_lot::Mutex;
use rayon::prelude::*;
use vfs::FileRef;

pub struct FileParseTree<L: LangExt> {
	pub path: PathBuf,
	pub inner: ParseTree<L>,
}

impl<L: LangExt> std::ops::Deref for FileParseTree<L> {
	type Target = ParseTree<L>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

/// For parsing from VileTech's [virtual file system](vfs).
#[derive(Debug)]
pub struct IncludeTree(HashMap<PathBuf, Result<GreenNode, parse::Error>>);

impl IncludeTree {
	/// Traverses a VFS directory, parsing every text file with the extension "lith".
	/// Panics if `root` is not a directory.
	#[must_use]
	pub fn new(root: FileRef) -> Self {
		fn recur(fref: FileRef, files: &Mutex<HashMap<PathBuf, Result<GreenNode, parse::Error>>>) {
			fref.children().unwrap().par_bridge().for_each(|child| {
				if child.is_dir() {
					recur(child, files);
				}

				if !child.is_text() {
					return;
				}

				if !child.path_extension().is_some_and(|ext| ext == "lith") {
					return;
				}

				let result = parse::file(child.read_str());
				files.lock().insert(child.path().to_path_buf(), result);
			});
		}

		assert!(
			root.is_dir(),
			"`IncludeTree::new` expects a virtual directory"
		);

		let ret = Mutex::new(HashMap::new());
		recur(root, &ret);
		Self(ret.into_inner())
	}

	#[must_use]
	pub fn any_errors(&self) -> bool {
		self.0.values().any(|result| result.is_err())
	}

	pub fn drain_errors(&mut self) -> impl Iterator<Item = (PathBuf, parse::Error)> + '_ {
		self.0
			.drain()
			.filter_map(|(path, result)| result.err().map(|err| (path, err)))
	}

	pub fn into_inner(self) -> impl Iterator<Item = (PathBuf, GreenNode)> {
		assert!(!self.any_errors(), "encountered one or more parse errors");

		self.0
			.into_iter()
			.map(|(path, result)| (path, result.unwrap()))
	}
}

/// Thin, strongly-typed wrapper around [`compile::ContainerSource`].
#[repr(transparent)]
pub struct ContainerSource<L: LangExt>(compile::ContainerSource, PhantomData<L>);

impl<L: LangExt> ContainerSource<L> {
	#[must_use]
	pub fn new(path: PathBuf, ptree: ParseTree<L>) -> Self {
		assert!(
			!ptree.any_errors(),
			"encountered one or more parsing errors"
		);

		Self(
			compile::ContainerSource {
				path,
				root: ptree.into_inner(),
			},
			PhantomData,
		)
	}
}

impl<L: LangExt> std::ops::Deref for ContainerSource<L> {
	type Target = compile::ContainerSource;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// A superset of [`compile::LibBuilder`].
pub struct LibBuilder {
	pub inner: compile::LibBuilder,
	// (G)ZDoom
	pub decorate: Vec<ContainerSource<Decorate>>,
	pub zscript: Vec<ContainerSource<ZScript>>,
}

impl LibBuilder {
	#[must_use]
	fn finish(self) -> LibSource {
		let subset = self.inner.finish();

		LibSource {
			_name: subset.name,
			_version: subset.version,
			_jit: subset.jit,
			// SAFETY: `ContainerSource<L>` is `repr(transparent)` over
			// `compile::ContainerSource`.
			lith: unsafe { std::mem::transmute::<_, _>(subset.sources) },
			decorate: self.decorate,
			zscript: self.zscript,
		}
	}
}

/// A superset of [`compile::LibSource`].
pub(crate) struct LibSource {
	_name: String,
	_version: Version,
	_jit: Arc<JitModule>,

	lith: Vec<ContainerSource<Syn>>,
	decorate: Vec<ContainerSource<Decorate>>,
	zscript: Vec<ContainerSource<ZScript>>,
}

pub fn compile(builders: impl IntoIterator<Item = LibBuilder>) {
	let containers = DashMap::new();
	let gex = DashMap::new();

	for source in builders.into_iter().map(|b| b.finish()) {
		// Declare containers.

		for ctr_lith in &source.lith {
			containers.insert(ctr_lith.path.clone().into(), lir::Container::default());
		}

		for ctr_dec in &source.decorate {
			containers.insert(ctr_dec.path.clone().into(), lir::Container::default());
		}

		for ctr_zs in &source.zscript {
			containers.insert(ctr_zs.path.clone().into(), lir::Container::default());
		}

		// Pass 1: declare container-level types.

		[lith::pass1, decorate::pass1, zscript::pass1]
			.par_iter()
			.for_each(|function| {
				let pass1 = Pass1 {
					src: &source,
					_containers: &containers,
				};

				function(pass1);
			});

		// Pass 2: check container-level type declarations.
		// Pass 3: declare functions and data items.

		[lith::pass3, decorate::pass3, zscript::pass3]
			.par_iter()
			.for_each(|function| {
				let pass3 = Pass3 {
					_src: &source,
					_containers: &containers,
					_gex: &gex,
				};

				function(pass3);
			});

		// Pass 4: check functions and data item declarations.
		// Pass 5: check function bodies and data initializers. Lower to LIR.
	}
}

// Pass 1 //////////////////////////////////////////////////////////////////////

pub(self) struct Pass1<'s> {
	src: &'s LibSource,
	_containers: &'s DashMap<Arc<Path>, lir::Container>,
}

// Pass 3 //////////////////////////////////////////////////////////////////////

pub(self) struct Pass3<'s> {
	_src: &'s LibSource,
	_containers: &'s DashMap<Arc<Path>, lir::Container>,
	/// When (G)ZDoom attempts to look up a symbol using just its unqualified name
	/// ASCII case-insensitively, it goes through the global export table ("GEX").
	_gex: &'s DashMap<ZName, Vec<(Arc<Path>, lir::Name)>>,
}

impl Pass3<'_> {
	pub(self) fn _gex_decl(&self, name: lir::Name, modpath: Arc<Path>) {
		let zname = ZName::from(&name);

		match self._gex.entry(zname) {
			dashmap::mapref::entry::Entry::Occupied(mut occ) => {
				occ.get_mut().push((modpath, name));
			}
			dashmap::mapref::entry::Entry::Vacant(vac) => {
				let mut syms = vac.insert(vec![]);
				syms.push((modpath, name));
			}
		}
	}
}

// Details /////////////////////////////////////////////////////////////////////

/// A counterpart to [`lir::Name`] for (G)ZDoom,
/// with ASCII case-insensitive equality comparison and hashing.
#[derive(Debug, Eq)]
pub(self) struct ZName(lir::Name);

impl PartialEq for ZName {
	fn eq(&self, other: &Self) -> bool {
		self.0.as_str().eq_ignore_ascii_case(other.0.as_str())
	}
}

impl std::hash::Hash for ZName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		for c in self.0.as_str().chars() {
			c.to_ascii_lowercase().hash(state);
		}
	}
}

impl std::borrow::Borrow<str> for ZName {
	fn borrow(&self) -> &str {
		self.0.as_str()
	}
}

impl From<&lir::Name> for ZName {
	fn from(value: &lir::Name) -> Self {
		Self(value.clone())
	}
}
