//! Parts of the frontend not concerned with [parsing] or [lexing].
//!
//! [parsing]: crate::parse
//! [lexing]: crate::syn

mod deco;
mod vzs;
mod zs;

use crossbeam::atomic::AtomicCell;
use doomfront::{
	rowan::{TextRange, TextSize},
	zdoom::zscript,
	ParseTree,
};
use parking_lot::Mutex;

use rayon::prelude::*;

use crate::{
	compile::{
		self,
		intern::{NsName, SymbolIx},
		symbol::{DefIx, Definition, Location, Symbol, SymbolKind},
		Compiler, LibSource, Scope,
	},
	inctree::SourceKind,
};

pub fn declare_symbols(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Declaration);
	debug_assert!(!compiler.any_errors());

	let mut namespaces = vec![];
	namespaces.resize_with(compiler.sources.len(), Scope::default);
	let namespaces = Mutex::new(namespaces);

	declaration_pass(
		compiler,
		&namespaces,
		|_, _, _| {},
		zs::declare_symbols_early,
	);

	// Give the current symbol tables to the compiler so that the next pass
	// can look up the early pass symbols across namespaces.
	compiler.namespaces = namespaces.into_inner();
	let mut namespaces = vec![];
	namespaces.resize_with(compiler.sources.len(), Scope::default);
	let namespaces = Mutex::new(namespaces);

	declaration_pass(
		compiler,
		&namespaces,
		vzs::declare_symbols,
		zs::declare_symbols,
	);

	// Now, merge the early pass with the main pass.

	let mut namespaces = std::mem::replace(&mut compiler.namespaces, namespaces.into_inner());
	debug_assert_eq!(namespaces.len(), compiler.namespaces.len());

	namespaces
		.iter_mut()
		.zip(compiler.namespaces.iter_mut())
		.par_bridge()
		.for_each(|(ns_early, ns_compiler)| {
			for kvp in ns_early.drain() {
				let overwritten = ns_compiler.insert(kvp.0, kvp.1);
				debug_assert!(overwritten.is_none());
			}
		});

	compiler.stage = compile::Stage::Semantic;

	if compiler.any_errors() {
		compiler.failed = true;
	}
}

fn declaration_pass(
	compiler: &Compiler,
	namespaces: &Mutex<Vec<Scope>>,
	vzs_function: fn(&DeclContext, &mut Scope, &crate::ParseTree),
	zs_function: fn(&DeclContext, &mut Scope, &ParseTree<zscript::Syn>),
) {
	compiler
		.sources
		.par_iter()
		.enumerate()
		.for_each(|(i, libsrc)| {
			let mut namespace = Scope::default();

			for (ii, pfile) in libsrc.inctree.files.iter().enumerate() {
				let mut ctx = DeclContext {
					compiler,
					lib: libsrc,
					path: pfile.path(),
					lib_ix: i as u16,
					file_ix: ii as u16,
					zscript: false,
				};

				match pfile.inner() {
					SourceKind::Vzs(ptree) => {
						vzs_function(&ctx, &mut namespace, ptree);
					}
					SourceKind::Zs(ptree) => {
						ctx.zscript = true;
						zs_function(&ctx, &mut namespace, ptree);
					}
				}
			}

			namespaces.lock()[i] = namespace;
		});
}

#[derive(Debug)]
pub(self) struct DeclContext<'c> {
	pub(self) compiler: &'c Compiler,
	pub(self) lib: &'c LibSource,
	pub(self) path: &'c str,
	pub(self) lib_ix: u16,
	pub(self) file_ix: u16,
	pub(self) zscript: bool,
}

impl DeclContext<'_> {
	/// If `Err` is returned, it contains the index
	/// to the symbol that would have been overwritten.
	pub(self) fn declare(
		&self,
		outer: &mut Scope,
		nsname: NsName,
		span: TextRange,
		short_end: TextSize,
		kind: SymbolKind,
		scope: Scope,
	) -> Result<SymbolIx, SymbolIx> {
		use std::collections::hash_map;

		let symbol = Symbol {
			location: Some(Location {
				lib_ix: self.lib_ix,
				file_ix: self.file_ix,
				span,
				short_end,
			}),
			kind,
			zscript: self.zscript,
			scope,
			definition: AtomicCell::new(DefIx::None),
		};

		match outer.entry(nsname) {
			hash_map::Entry::Vacant(vac) => {
				let ix = SymbolIx(self.symbols.push(symbol) as u32);
				vac.insert(ix);
				Ok(ix)
			}
			hash_map::Entry::Occupied(occ) => Err(*occ.get()),
		}
	}
}

impl std::ops::Deref for DeclContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}
