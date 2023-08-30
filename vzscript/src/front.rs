//! Parts of the frontend not concerned with [parsing] or [lexing].
//!
//! [parsing]: crate::parse
//! [lexing]: crate::syn

mod deco;
mod vzs;
mod zs;

use std::sync::{
	atomic::{self, AtomicBool},
	Arc,
};

use arc_swap::ArcSwap;
use crossbeam::{queue::SegQueue, utils::Backoff};
use doomfront::rowan::{GreenNode, TextRange};
use parking_lot::Mutex;
use petgraph::prelude::DiGraph;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;

use crate::{
	compile::{self, Compiler, NameKey, PathKey, SymbolKey},
	inctree::SourceKind,
	issue::{self, FileSpan, Issue, IssueLevel},
	rti,
	tsys::{ClassType, EnumType, FuncType, StructType, TypeDef, TypeHandle, UnionType},
};

#[derive(Debug)]
pub(crate) enum Symbol {
	Class {
		typedef: TypeHandle<ClassType>,
		scope: Scope,
	},
	Enum {
		typedef: TypeHandle<EnumType>,
		scope: Scope,
	},
	Function {
		typedef: TypeHandle<FuncType>,
		body: Option<DiGraph<Scope, ()>>,
	},
	Mixin {
		green: GreenNode,
	},
	Struct {
		typedef: TypeHandle<StructType>,
		scope: Scope,
	},
	Union {
		typedef: TypeHandle<UnionType>,
		scope: Scope,
	},
	Value {
		typedef: rti::Handle<TypeDef>,
		mutable: bool,
	},
	Undefined {
		location: Location,
		kind: UndefKind,
		scope: Scope,
	},
}

impl Symbol {
	#[must_use]
	pub(crate) fn location(&self) -> Location {
		todo!();
		todo!()
	}
}

#[derive(Debug)]
pub(crate) struct Location {
	pub(crate) file: PathKey,
	pub(crate) span: TextRange,
}

#[derive(Debug)]
pub(crate) enum UndefKind {
	Class,
	Enum,
	Function,
	Struct,
	Union,
	Value { mutable: bool },
}

pub(crate) type SymbolPtr = ArcSwap<Symbol>;
pub(crate) type Scope = FxHashMap<NameKey, SymbolKey>;

impl From<Symbol> for SymbolPtr {
	fn from(value: Symbol) -> Self {
		ArcSwap::new(Arc::new(value))
	}
}

pub fn declare_symbols(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Resolution);
	assert!(compiler.cur_lib < compiler.sources.len());
	assert!(!compiler.any_errors());

	let global = Mutex::new(Scope::default());
	let sym_q = SegQueue::new();
	let done = AtomicBool::new(false);
	let libsrc = &compiler.sources[compiler.cur_lib];

	libsrc.inctree.files.par_iter().for_each(|pfile| {
		// TODO: Regardless of what concurrent collection is being used here,
		// this may still be subject to thundering herd effects. Investigate.
		let _ = compiler.paths.intern(pfile.path());
	});

	// TODO: Using a concurrent queue here instead of directly inserting into
	// a concurrent global symbol map is weird, but the latter penalizes all
	// later operations. Either all scopes have to be concurrent or lookups
	// have to branch to check if dealing with a concurrent scope and still
	// pay the extra price of lookup in the concurrent scope.
	// Worth investigating in the future.

	rayon::join(
		|| {
			let mut guard = global.lock();
			let backoff = Backoff::new();

			while !done.load(atomic::Ordering::Acquire) {
				backoff.snooze();

				while let Some((name_k, symbol)) = sym_q.pop() {
					let Some((symbol, other_k)) = compiler.declare(
						&mut guard,
						name_k, symbol
					) else {
						continue;
					};

					let location = symbol.location();

					let path_e = compiler.paths.resolve(location.file).unwrap();
					let name_e = compiler.names.resolve(name_k).unwrap();
					let path: &str = path_e.as_ref();
					let name: &str = name_e.as_ref();

					let other = compiler.get_symbol(other_k);
					let o_location = other.load().location();
					let o_path_e = compiler.paths.resolve(o_location.file).unwrap();
					let o_path: &str = o_path_e.as_ref();

					compiler.raise(Issue {
						id: FileSpan::new(path, location.span),
						level: IssueLevel::Error(issue::Error::Redeclare),
						message: format!("attempt to re-declare `{}`", name),
						label: Some(issue::Label {
							id: FileSpan::new(o_path, o_location.span),
							message: "original declaration is here".to_string(),
						}),
					});
				}
			}
		},
		|| {
			libsrc.inctree.files.par_iter().for_each(|pfile| {
				let path = pfile.path();

				let ctx = DeclContext {
					compiler,
					sym_q: &sym_q,
					path_k: compiler.paths.intern(path),
				};

				match pfile.inner() {
					SourceKind::Vzs(ptree) => {
						vzs::declare_symbols(ctx, ptree);
					}
					SourceKind::Zs(ptree) => {
						zs::declare_symbols(ctx, ptree);
					}
				}
			});

			if let Some(_deco) = &libsrc.decorate {
				// TODO
			}

			done.store(true, atomic::Ordering::Release);
			std::thread::park();
		},
	);

	compiler.global = global.into_inner();
	compiler.stage = compile::Stage::Resolution;
}

pub fn resolve_names(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Resolution);
	assert!(compiler.cur_lib < compiler.sources.len());
	assert!(!compiler.any_errors());

	compiler.stage = compile::Stage::Checking;
}

pub fn semantic_checks(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Checking);
	assert!(compiler.cur_lib < compiler.sources.len());
	assert!(!compiler.any_errors());

	compiler.cur_lib += 1;
	compiler.stage = compile::Stage::Declaration;
}

#[derive(Debug, Clone, Copy)]
pub(self) struct DeclContext<'c> {
	pub(self) compiler: &'c Compiler,
	pub(self) sym_q: &'c SegQueue<(NameKey, Symbol)>,
	pub(self) path_k: PathKey,
}

impl std::ops::Deref for DeclContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}
