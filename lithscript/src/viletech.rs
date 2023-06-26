//! A decoupled wing of LithScript for serving the VileTech Engine's specific needs.

mod cvarinfo;
mod decorate;
mod language;
mod zscript;

use std::{collections::HashMap, hash::Hasher, path::PathBuf};

use dashmap::DashMap;
use doomfront::{
	rowan::GreenNode,
	zdoom::{
		cvarinfo::Syn as CVarInfo, decorate::Syn as Decorate, inctree::IncludeTree as ZIncludeTree,
		language::Syn as Language, zscript::Syn as ZScript,
	},
	ParseTree,
};
use parking_lot::Mutex;
use rayon::prelude::*;
use util::rstring::RString;
use vfs::FileRef;

use crate::parse;

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
}

#[derive(Debug)]
pub struct LibSource {
	lith: Option<IncludeTree>,
	// (G)ZDoom
	cvarinfo: Vec<ParseTree<CVarInfo>>,
	decorate: Option<ZIncludeTree<Decorate>>,
	language: Vec<ParseTree<Language>>,
	zscript: Option<ZIncludeTree<ZScript>>,
}

pub(self) type GlobalExportTable = DashMap<QualName, ()>;

#[derive(Debug)]
pub(self) enum QualName {
	_Cased(RString),
	_Caseless(RString),
}

impl PartialEq for QualName {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::_Cased(l0), Self::_Cased(r0)) => l0 == r0,
			(Self::_Caseless(l0), Self::_Caseless(r0)) => l0.eq_ignore_ascii_case(r0),
			_ => false,
		}
	}
}

impl Eq for QualName {}

impl std::hash::Hash for QualName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			QualName::_Cased(string) => string.hash(state),
			QualName::_Caseless(string) => string
				.chars()
				.for_each(|c| c.to_ascii_lowercase().hash(state)),
		}
	}
}

/// Panics if any parse errors are detected in `sources`.
pub fn compile(sources: impl IntoIterator<Item = LibSource>) {
	let gex = GlobalExportTable::new();
	let sources = sources.into_iter().collect::<Vec<_>>();

	// Step 1: pass over everything that becomes a Lith module to build the global
	// export table ("GEX"). Most languages covered here have nothing resembling
	// imperative code, but we also want to do this before lowering to LIR, so we
	// often have to transpile declarative code to function and data definitions.

	for source in &sources {
		[
			export,
			cvarinfo::export,
			decorate::export,
			language::export,
			zscript::export,
		]
		.par_iter()
		.for_each(|export_fn| {
			export_fn(source, &gex);
		})
	}
}

fn export(source: &LibSource, _: &GlobalExportTable) {
	let Some(inctree) = &source.lith else { return; };
	assert!(!inctree.any_errors())
}
