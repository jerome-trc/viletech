//! Parts of the frontend not concerned with [parsing] or [lexing].
//!
//! [parsing]: crate::parse
//! [lexing]: crate::syn

mod deco;
mod vzs;
mod zs;

use parking_lot::Mutex;

use rayon::prelude::*;

use crate::{
	compile::{self, intern::PathIx, Compiler, Scope},
	inctree::SourceKind,
};

pub fn declare_symbols(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Declaration);
	debug_assert!(!compiler.any_errors());

	let mut namespaces = vec![];
	namespaces.resize_with(compiler.sources.len(), Scope::default);
	let namespaces = Mutex::new(namespaces);

	compiler
		.sources
		.par_iter()
		.enumerate()
		.for_each(|(i, libsrc)| {
			let mut namespace = Scope::default();

			for pfile in &libsrc.inctree.files {
				let path_ix = compiler.paths.intern(pfile.path());

				let ctx = DeclContext {
					compiler,
					path: pfile.path(),
					path_ix,
				};

				match pfile.inner() {
					SourceKind::Vzs(ptree) => {
						vzs::declare_symbols(&ctx, &mut namespace, ptree);
					}
					SourceKind::Zs(ptree) => {
						zs::declare_symbols(&ctx, &mut namespace, ptree);
					}
				}
			}

			namespaces.lock()[i] = namespace;
		});

	compiler.namespaces = namespaces.into_inner();
	compiler.stage = compile::Stage::Semantic;

	if compiler.any_errors() {
		compiler.failed = true;
	}
}

#[derive(Debug)]
pub(self) struct DeclContext<'c> {
	pub(self) compiler: &'c Compiler,
	pub(self) path: &'c str,
	pub(self) path_ix: PathIx,
}

impl std::ops::Deref for DeclContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}
