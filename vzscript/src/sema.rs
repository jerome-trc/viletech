pub(crate) mod vzs;
pub(crate) mod zs;

use crossbeam::utils::Backoff;
use doomfront::rowan::cursor::SyntaxNode;
use rayon::prelude::*;
use smallvec::SmallVec;

use crate::{
	compile::{
		self,
		intern::NsName,
		symbol::{DefIx, Location, Symbol},
		Compiler, Scope,
	},
	inctree::{ParsedFile, SourceKind},
	rti,
	tsys::TypeDef,
	vir,
};

pub fn sema(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Semantic);
	assert!(!compiler.failed);

	for (i, libsrc) in compiler.sources.iter().enumerate() {
		// TODO: Investigate if it's faster to clone the green node for each symbol.
		// Maybe too much atomic contention?
		libsrc
			.inctree
			.files
			.par_iter()
			.enumerate()
			.for_each(|(ii, pfile)| {
				match pfile.inner() {
					SourceKind::Vzs(ptree) => {
						let cursor = ptree.cursor();

						for_each_symbol(compiler, pfile, i, ii, |ctx, symbol| {
							vzs::define(ctx, &cursor, symbol)
						});
					}
					SourceKind::Zs(ptree) => {
						let cursor = ptree.cursor();

						for_each_symbol(compiler, pfile, i, ii, |ctx, symbol| {
							zs::define(ctx, &cursor, symbol)
						});
					}
				};
			});
	}

	compiler.stage = compile::Stage::CodeGen;

	if compiler.any_errors() {
		compiler.failed = true;
	}
}

fn for_each_symbol(
	compiler: &Compiler,
	pfile: &ParsedFile,
	lib_ix: usize,
	file_ix: usize,
	function: impl Fn(&SemaContext, &Symbol) -> DefIx,
) {
	for symbol in compiler.symbols.iter() {
		let Some(location) = symbol.location else {
			continue;
		};

		if location.lib_ix as usize != lib_ix {
			continue;
		}

		if location.file_ix as usize != file_ix {
			continue;
		}

		if symbol
			.definition
			.compare_exchange(DefIx::None, DefIx::Pending)
			.is_err()
		{
			continue;
		}

		let ctx = SemaContext {
			compiler,
			location,
			path: pfile.path(),
			zscript: symbol.zscript,
		};

		symbol.definition.store(function(&ctx, symbol));
	}
}

#[derive(Debug)]
pub(self) struct SemaContext<'c> {
	pub(self) compiler: &'c Compiler,
	pub(self) location: Location,
	pub(self) path: &'c str,
	pub(self) zscript: bool,
}

impl std::ops::Deref for SemaContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}

/// If `symbol` is defined, this is a no-op.
/// If `symbol` is pending a definition, use exponential backoff to wait until
/// that definition is complete.
/// If `symbol` is undefined, lazily provide a definition for it.
#[must_use]
pub(self) fn require(ctx: &SemaContext, root: SyntaxNode, symbol: &Symbol) -> DefIx {
	if symbol
		.definition
		.compare_exchange(DefIx::None, DefIx::Pending)
		.is_ok()
	{
		let new_ctx = SemaContext {
			location: symbol.location.unwrap(),
			zscript: symbol.zscript,
			..*ctx
		};

		let def_ix = if symbol.zscript {
			let root = root.into();
			zs::define(&new_ctx, &root, symbol)
		} else {
			let root = root.into();
			vzs::define(&new_ctx, &root, symbol)
		};

		symbol.definition.store(def_ix);
		return def_ix;
	}

	let backoff = Backoff::new();
	let mut definition = symbol.definition.load();

	while definition == DefIx::Pending {
		backoff.snooze();
		definition = symbol.definition.load();
	}

	definition
}

#[derive(Debug)]
pub(self) struct ConstEval {
	/// `None` if the type cannot be known, such as when compile-time-evaluating
	/// a null pointer literal.
	pub(self) typedef: Option<rti::Handle<TypeDef>>,
	pub(self) ir: vir::Node,
}

/// The output of a compile-time evaluated expression.
///
/// Type definition handles will be `None` wherever type inference fails.
#[derive(Debug, Clone)]
pub(crate) enum CEval {
	SelfPtr {
		typedef: Option<rti::Handle<TypeDef>>,
	},
	SuperPtr {
		typedef: Option<rti::Handle<TypeDef>>,
	},
	Type {
		def: rti::Handle<TypeDef>,
	},
	Value {
		typedef: Option<rti::Handle<TypeDef>>,
		value: SmallVec<[vir::Immediate; 1]>,
	},
	Error,
}

pub(crate) type CEvalVec = SmallVec<[CEval; 1]>;

#[derive(Debug, Clone)]
pub(self) struct ScopeStack<'s>(Vec<&'s Scope>, &'s Compiler);

impl<'s> std::ops::Deref for ScopeStack<'s> {
	type Target = Vec<&'s Scope>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::ops::DerefMut for ScopeStack<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl ScopeStack<'_> {
	#[must_use]
	pub(self) fn lookup(&self, nsname: NsName) -> Option<&Symbol> {
		self.0
			.iter()
			.rev()
			.find_map(|scope| scope.get(&nsname).map(|&sym_ix| self.1.symbol(sym_ix)))
	}
}
