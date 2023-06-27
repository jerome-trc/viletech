//! A decoupled wing of LithScript for serving the VileTech Engine's specific needs.

mod lith;

mod decorate;
mod zscript;

use std::{
	collections::HashMap,
	hash::Hasher,
	marker::PhantomData,
	mem::MaybeUninit,
	path::{Path, PathBuf},
	sync::Arc,
};

use crate::{issue::Issue, lir, parse, Version};
use cranelift::prelude::settings::OptLevel;
use cranelift_jit::{JITBuilder, JITModule};
use dashmap::DashMap;
use doomfront::{
	rowan::GreenNode,
	zdoom::{decorate::Syn as Decorate, zscript::Syn as ZScript},
	LangExt, ParseTree,
};
use parking_lot::Mutex;
use rayon::prelude::*;
use util::rstring::RString;
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

pub struct ModuleBuilder<L: LangExt> {
	path: PathBuf,
	root: GreenNode,
	jit: JITBuilder,
	phantom: PhantomData<L>,
}

impl<L: LangExt> ModuleBuilder<L> {
	#[must_use]
	pub fn new(ptree: ParseTree<L>, path: PathBuf, opt: OptLevel) -> Self {
		assert!(!ptree.any_errors(), "one or more parse errors detected");

		let o_lvl = match opt {
			OptLevel::None => "none",
			OptLevel::Speed => "speed",
			OptLevel::SpeedAndSize => "speed_and_size",
		};

		Self {
			path,
			root: ptree.into_inner(),
			jit: JITBuilder::with_flags(
				&[
					("use_colocated_libcalls", "false"),
					("is_pic", "false"),
					("opt_level", o_lvl),
				],
				cranelift_module::default_libcall_names(),
			)
			.expect("JIT module builder creation failed"),
			phantom: PhantomData,
		}
	}

	pub fn symbols<I, N>(&mut self, symbols: I)
	where
		I: IntoIterator<Item = (N, *const u8)>,
		N: Into<String>,
	{
		self.jit.symbols(symbols);
	}

	pub fn finish(self) -> ModuleSource<L> {
		ModuleSource {
			path: self.path,
			root: self.root,
			_jit: Arc::new(JitModule(Mutex::new(MaybeUninit::new(JITModule::new(
				self.jit,
			))))),
			phantom: PhantomData,
		}
	}
}

pub struct ModuleSource<L: LangExt> {
	path: PathBuf,
	root: GreenNode,
	_jit: Arc<JitModule>,
	phantom: PhantomData<L>,
}

pub struct LibSource {
	// Metadata
	pub name: String,
	pub version: Version,

	pub lith: Vec<ModuleSource<crate::Syn>>,

	// (G)ZDoom
	pub decorate: Vec<ModuleSource<Decorate>>,
	pub zscript: Vec<ModuleSource<ZScript>>,
}

pub struct Precompile {
	_ir: Vec<(lir::Module, Arc<JitModule>)>,
	_issues: Vec<Issue>,
}

pub fn compile(sources: impl IntoIterator<Item = LibSource>) {
	let modules = DashMap::new();
	let gex = DashMap::new();
	let sources = sources.into_iter().collect::<Vec<_>>();

	for source in &sources {
		// Declare modules.

		for mod_lith in &source.lith {
			modules.insert(mod_lith.path.clone().into(), lir::Module::default());
		}

		for mod_dec in &source.decorate {
			modules.insert(mod_dec.path.clone().into(), lir::Module::default());
		}

		for mod_zs in &source.zscript {
			modules.insert(mod_zs.path.clone().into(), lir::Module::default());
		}

		// Pass 1: declare module-level types.

		[lith::pass1, decorate::pass1, zscript::pass1]
			.par_iter()
			.for_each(|function| {
				let pass1 = Pass1 {
					src: source,
					_modules: &modules,
				};

				function(pass1);
			});

		// Pass 2: check module-level type declarations.
		// Pass 3: declare functions and data items.

		[lith::pass3, decorate::pass3, zscript::pass3]
			.par_iter()
			.for_each(|function| {
				let pass3 = Pass3 {
					_src: source,
					_modules: &modules,
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
	_modules: &'s DashMap<Arc<Path>, lir::Module>,
}

// Pass 3 //////////////////////////////////////////////////////////////////////

pub(self) struct Pass3<'s> {
	_src: &'s LibSource,
	_modules: &'s DashMap<Arc<Path>, lir::Module>,
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
pub(self) enum ZName {
	Var(RString),
	Func(RString),
	Module(RString),
}

impl PartialEq for ZName {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Var(lhs), Self::Var(rhs)) => lhs.eq_ignore_ascii_case(rhs),
			(Self::Func(lhs), Self::Func(rhs)) => lhs.eq_ignore_ascii_case(rhs),
			(Self::Module(lhs), Self::Module(rhs)) => lhs.eq_ignore_ascii_case(rhs),
			_ => false,
		}
	}
}

impl std::hash::Hash for ZName {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let string = match self {
			ZName::Var(string) | ZName::Func(string) | ZName::Module(string) => string,
		};

		for c in string.chars() {
			c.to_ascii_lowercase().hash(state);
		}
	}
}

impl std::borrow::Borrow<str> for ZName {
	fn borrow(&self) -> &str {
		match self {
			ZName::Var(string) | ZName::Func(string) | ZName::Module(string) => string.as_ref(),
		}
	}
}

impl From<&lir::Name> for ZName {
	fn from(value: &lir::Name) -> Self {
		match value {
			lir::Name::Var(string) => Self::Var(string.clone()),
			lir::Name::Func(string) => Self::Func(string.clone()),
			lir::Name::Module(string) => Self::Module(string.clone()),
		}
	}
}

/// Ensures proper JIT code de-allocation (from behind an [`Arc`]).
#[derive(Debug)]
pub(crate) struct JitModule(pub(crate) Mutex<MaybeUninit<JITModule>>);

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			std::mem::replace(&mut self.0, Mutex::new(MaybeUninit::uninit()))
				.into_inner()
				.assume_init()
				.free_memory();
		}
	}
}

// TODO: Prove that this is safe when codegen machinery is in working order.
unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}
