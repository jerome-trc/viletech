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
use parking_lot::{Mutex, RwLock};
use petgraph::prelude::DiGraph;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;

use crate::{
	compile::{self, Compiler, LibSource, NameKey, PathKey, SymbolKey},
	inctree::SourceKind,
	issue::{self, FileSpan, Issue, IssueLevel},
	rti,
	tsys::{ClassType, EnumType, FuncType, StructType, TypeDef, TypeHandle, UnionType},
};

#[derive(Debug)]
pub(crate) enum Symbol {
	Class {
		location: Location,
		typedef: TypeHandle<ClassType>,
		scope: Scope,
	},
	Enum {
		location: Location,
		typedef: TypeHandle<EnumType>,
		scope: Scope,
	},
	FlagDef {
		location: Location,
	},
	Function {
		location: Location,
		typedef: TypeHandle<FuncType>,
		body: Option<DiGraph<Scope, ()>>,
	},
	Mixin {
		location: Location,
		green: GreenNode,
	},
	Primitive {
		location: Location,
		scope: Scope,
	},
	Property {
		location: Location,
	},
	Struct {
		location: Location,
		typedef: TypeHandle<StructType>,
		scope: Scope,
	},
	Union {
		location: Location,
		typedef: TypeHandle<UnionType>,
		scope: Scope,
	},
	Value {
		location: Location,
		typedef: rti::Handle<TypeDef>,
		mutable: bool,
	},
	Undefined {
		location: Location,
		kind: UndefKind,
		scope: RwLock<Scope>,
	},
}

impl Symbol {
	#[must_use]
	pub(crate) fn location(&self) -> Location {
		match self {
			Symbol::Class { location, .. }
			| Symbol::Enum { location, .. }
			| Symbol::FlagDef { location, .. }
			| Symbol::Function { location, .. }
			| Symbol::Mixin { location, .. }
			| Symbol::Primitive { location, .. }
			| Symbol::Property { location, .. }
			| Symbol::Struct { location, .. }
			| Symbol::Union { location, .. }
			| Symbol::Value { location, .. }
			| Symbol::Undefined { location, .. } => *location,
		}
	}
}

impl Clone for Symbol {
	fn clone(&self) -> Self {
		let Self::Undefined { location, kind, scope } = self else {
			panic!()
		};

		Self::Undefined {
			location: *location,
			kind: *kind,
			scope: RwLock::new(scope.read().clone()),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Location {
	pub(crate) file: PathKey,
	pub(crate) span: TextRange,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UndefKind {
	Class,
	Enum,
	FlagDef,
	Function,
	Property,
	Struct,
	Union,
	Value { comptime: bool, mutable: bool },
}

pub(crate) type SymbolPtr = ArcSwap<Symbol>;
pub(crate) type Scope = FxHashMap<NameKey, SymbolKey>;

impl From<Symbol> for SymbolPtr {
	fn from(value: Symbol) -> Self {
		ArcSwap::new(Arc::new(value))
	}
}

pub fn declare_symbols(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Declaration);
	debug_assert!(!compiler.any_errors());

	let sym_q = SegQueue::new();
	let done = AtomicBool::new(false);

	let (global, ()) = rayon::join(
		|| global_symbol_funnel(compiler, &sym_q, &done),
		|| walk_sources(compiler, &sym_q, &done),
	);

	compiler.global = global;
	compiler.stage = compile::Stage::Finalize;
}

// TODO: Using a concurrent queue here instead of directly inserting into
// a concurrent global symbol map is weird, but the latter would penalize
// all later operations. Worth investigating in the future.
fn global_symbol_funnel(
	compiler: &Compiler,
	queue: &SegQueue<(NameKey, Symbol)>,
	done: &AtomicBool,
) -> Scope {
	let mut global = Scope::default();
	let backoff = Backoff::new();

	while !done.load(atomic::Ordering::Acquire) {
		backoff.snooze();

		while let Some((name_k, symbol)) = queue.pop() {
			let Err((symbol, other_k)) = compiler.declare(
				&mut global,
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

	global
}

fn walk_sources(compiler: &Compiler, sym_q: &SegQueue<(NameKey, Symbol)>, done: &AtomicBool) {
	for libsrc in &compiler.sources {
		libsrc.inctree.files.par_iter().for_each(|pfile| {
			// TODO: Regardless of what concurrent collection is being used here,
			// this may still be subject to thundering herd effects. Investigate.
			let _ = compiler.paths.intern(pfile.path());
		});

		libsrc.inctree.files.par_iter().for_each(|pfile| {
			let path = pfile.path();

			let ctx = DeclContext {
				compiler,
				sym_q,
				path,
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
	}

	done.store(true, atomic::Ordering::Release);
}

pub fn finalize(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Finalize);
	assert!(!compiler.any_errors());
}

#[derive(Debug, Clone, Copy)]
pub(self) struct DeclContext<'c> {
	pub(self) compiler: &'c Compiler,
	pub(self) sym_q: &'c SegQueue<(NameKey, Symbol)>,
	pub(self) path: &'c str,
	pub(self) path_k: PathKey,
}

impl std::ops::Deref for DeclContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}
