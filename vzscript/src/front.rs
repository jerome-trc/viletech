//! Parts of the frontend not concerned with [parsing] or [lexing].
//!
//! [parsing]: crate::parse
//! [lexing]: crate::syn

mod deco;
mod vzs;
mod zs;

use std::{
	any::Any,
	sync::{
		atomic::{self, AtomicBool},
		Arc,
	},
};

use arc_swap::ArcSwap;
use crossbeam::{queue::SegQueue, utils::Backoff};
use doomfront::rowan::{GreenNode, TextRange};
use parking_lot::{Mutex, RwLock};
use petgraph::prelude::DiGraph;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;
use util::rstring::RString;

use crate::{
	compile::{self, Compiler, LibSource, Scope, SymbolPtr},
	inctree::SourceKind,
	issue::{self, FileSpan, Issue, IssueLevel},
	rti,
	tsys::{ClassType, EnumType, FuncType, StructType, TypeDef, TypeHandle, UnionType},
	vir,
	zname::ZName,
	FxDashMap,
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
		body: Option<DiGraph<vir::Block, ()>>,
	},
	Mixin {
		location: Location,
		green: GreenNode,
	},
	Primitive {
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
		kind: Undefined,
		scope: RwLock<Scope>,
	},
	Busy,
}

impl Symbol {
	#[must_use]
	pub(crate) fn location(&self) -> &Location {
		match self {
			Symbol::Class { location, .. }
			| Symbol::Enum { location, .. }
			| Symbol::FlagDef { location, .. }
			| Symbol::Function { location, .. }
			| Symbol::Mixin { location, .. }
			| Symbol::Property { location, .. }
			| Symbol::Struct { location, .. }
			| Symbol::Union { location, .. }
			| Symbol::Value { location, .. }
			| Symbol::Undefined { location, .. } => location,
			Symbol::Busy | Symbol::Primitive { .. } => panic!(),
		}
	}

	#[must_use]
	pub(crate) fn scope(&self) -> Option<&Scope> {
		match self {
			Symbol::Class { scope, .. }
			| Symbol::Enum { scope, .. }
			| Symbol::Primitive { scope, .. }
			| Symbol::Struct { scope, .. }
			| Symbol::Union { scope, .. } => Some(scope),
			Symbol::Busy
			| Symbol::Mixin { .. }
			| Symbol::FlagDef { .. }
			| Symbol::Function { .. }
			| Symbol::Property { .. }
			| Symbol::Undefined { .. }
			| Symbol::Value { .. } => None,
		}
	}

	#[must_use]
	pub(crate) fn is_undefined(&self) -> bool {
		matches!(self, Self::Undefined { .. })
	}

	#[must_use]
	pub(crate) fn is_busy(&self) -> bool {
		matches!(self, Self::Busy)
	}
}

impl Clone for Symbol {
	fn clone(&self) -> Self {
		let Self::Undefined { location, kind, scope } = self else {
			panic!()
		};

		Self::Undefined {
			location: location.clone(),
			kind: kind.clone(),
			scope: RwLock::new(scope.read().clone()),
		}
	}
}

#[derive(Debug, Clone)]
pub(crate) struct Location {
	pub(crate) file: RString,
	pub(crate) span: TextRange,
}

#[derive(Debug, Clone)]
pub(crate) enum Undefined {
	Class,
	Enum,
	FlagDef,
	Function,
	Property,
	Struct,
	Union,
	Value,
}

impl From<Symbol> for SymbolPtr {
	fn from(value: Symbol) -> Self {
		Arc::new(ArcSwap::from_pointee(value))
	}
}

pub fn declare_symbols(compiler: &mut Compiler) {
	assert!(!compiler.decl_done);
	debug_assert!(!compiler.any_errors());

	let sym_q = SegQueue::new();
	let done = AtomicBool::new(false);

	let (globals, ()) = rayon::join(
		|| global_symbol_funnel(compiler, &sym_q, &done),
		|| walk_sources(compiler, &sym_q, &done),
	);

	compiler.globals = globals;
	compiler.decl_done = true;
}

// TODO: Using a concurrent queue here instead of directly inserting into
// a concurrent global symbol map is weird, but the latter would penalize
// all later operations. Worth investigating in the future.
fn global_symbol_funnel(
	compiler: &Compiler,
	queue: &SegQueue<(ZName, Symbol)>,
	done: &AtomicBool,
) -> Scope {
	let mut global = Scope::default();
	let backoff = Backoff::new();

	while !done.load(atomic::Ordering::Acquire) {
		backoff.snooze();

		while let Some((iname, symbol)) = queue.pop() {
			let Err((symbol, other)) = compiler.declare(
				&mut global,
				iname.clone(), symbol
			) else {
				continue;
			};

			let other = other.load();
			let location = symbol.location();
			let o_location = other.location();

			compiler.raise([Issue {
				id: FileSpan::new(location.file.as_str(), location.span),
				level: IssueLevel::Error(issue::Error::Redeclare),
				message: format!("attempt to re-declare `{}`", iname),
				label: Some(issue::Label {
					id: FileSpan::new(o_location.file.as_str(), o_location.span),
					message: "original declaration is here".to_string(),
				}),
			}]);
		}
	}

	global
}

fn walk_sources(compiler: &Compiler, sym_q: &SegQueue<(ZName, Symbol)>, done: &AtomicBool) {
	for libsrc in &compiler.sources {
		libsrc.inctree.files.par_iter().for_each(|pfile| {
			// TODO: Regardless of what concurrent collection is being used here,
			// this may still be subject to thundering herd effects. Investigate.
			let _ = compiler.intern_path(pfile.path());
		});

		libsrc.inctree.files.par_iter().for_each(|pfile| {
			let ipath = compiler.intern_path(pfile.path());

			let ctx = DeclContext {
				compiler,
				sym_q,
				ipath: &ipath,
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

#[derive(Debug, Clone, Copy)]
pub(self) struct DeclContext<'c> {
	pub(self) compiler: &'c Compiler,
	pub(self) sym_q: &'c SegQueue<(ZName, Symbol)>,
	pub(self) ipath: &'c RString,
}

impl std::ops::Deref for DeclContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}
