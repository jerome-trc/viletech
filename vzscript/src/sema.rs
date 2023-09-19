pub(crate) mod vzs;
pub(crate) mod zs;

use std::sync::Arc;

use arc_swap::Guard;
use crossbeam::{atomic::AtomicCell, utils::Backoff};
use doomfront::rowan::cursor::SyntaxNode;
use parking_lot::RwLock;
use rayon::prelude::*;
use smallvec::SmallVec;

use crate::{
	compile::{
		self,
		intern::NsName,
		symbol::{DefStatus, Definition, Location, Symbol},
		Compiler, Scope,
	},
	inctree::{ParsedFile, SourceKind},
	rti,
	tsys::{PrimitiveType, TypeDef},
	vir, ArcGuard,
};

pub fn sema(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Semantic);
	assert!(!compiler.failed);

	let tcache = TypeCache {
		bool_t: vzs::define_primitive(compiler, "bool"),
		int32_t: vzs::define_primitive(compiler, "int32"),
		uint32_t: vzs::define_primitive(compiler, "uint32"),
		int64_t: vzs::define_primitive(compiler, "int64"),
		uint64_t: vzs::define_primitive(compiler, "uint64"),
		float32_t: vzs::define_primitive(compiler, "float32"),
		float64_t: vzs::define_primitive(compiler, "float64"),
		string_t: vzs::define_primitive(compiler, "string"),
		iname_t: vzs::define_primitive(compiler, "iname"),
	};

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

						for_each_symbol(compiler, &tcache, pfile, i, ii, |ctx, symbol| {
							vzs::define(ctx, &cursor, symbol)
						});
					}
					SourceKind::Zs(ptree) => {
						let cursor = ptree.cursor();

						for_each_symbol(compiler, &tcache, pfile, i, ii, |ctx, symbol| {
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
	tcache: &TypeCache,
	pfile: &ParsedFile,
	lib_ix: usize,
	file_ix: usize,
	function: impl Fn(&SemaContext, &Symbol) -> DefStatus,
) {
	for symbol in compiler.symbols.iter() {
		if symbol.location.lib_ix as usize != lib_ix {
			continue;
		}

		if symbol.location.file_ix as usize != file_ix {
			continue;
		}

		if symbol
			.status
			.compare_exchange(DefStatus::None, DefStatus::Pending)
			.is_err()
		{
			continue;
		}

		let ctx = SemaContext {
			compiler,
			tcache: Some(tcache),
			location: symbol.location,
			path: pfile.path(),
			zscript: symbol.zscript,
		};

		symbol.status.store(function(&ctx, symbol));
	}
}

#[derive(Debug)]
struct SemaContext<'c> {
	compiler: &'c Compiler,
	tcache: Option<&'c TypeCache>,
	location: Location,
	path: &'c str,
	zscript: bool,
}

impl SemaContext<'_> {
	/// If `symbol` is defined, this is a no-op.
	/// If `symbol` is pending a definition, use exponential backoff to wait until
	/// that definition is complete.
	/// If `symbol` is undefined, lazily provide a definition for it.
	#[must_use]
	fn require(&self, symbol: &Symbol) -> DefStatus {
		if symbol
			.status
			.compare_exchange(DefStatus::None, DefStatus::Pending)
			.is_ok()
		{
			let new_ctx = SemaContext {
				location: symbol.location,
				zscript: symbol.zscript,
				..*self
			};

			let libsrc = &self.sources[new_ctx.location.lib_ix as usize];
			let pfile = &libsrc.inctree.files[new_ctx.location.file_ix as usize];

			let status = match pfile.inner() {
				SourceKind::Vzs(ptree) => {
					let root = ptree.cursor();
					vzs::define(&new_ctx, &root, symbol)
				}
				SourceKind::Zs(ptree) => {
					let root = ptree.cursor();
					zs::define(&new_ctx, &root, symbol)
				}
			};

			symbol.status.store(status);
			return status;
		}

		let backoff = Backoff::new();
		let mut status = symbol.status.load();

		while status == DefStatus::Pending {
			backoff.snooze();
			status = symbol.status.load();
		}

		status
	}

	#[must_use]
	fn tcache(&self) -> &TypeCache {
		self.tcache.unwrap()
	}
}

impl std::ops::Deref for SemaContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}

#[derive(Debug)]
struct ConstEval {
	/// `None` if the type cannot be known, such as when compile-time-evaluating
	/// a null pointer literal.
	typedef: Option<rti::Handle<TypeDef>>,
	ir: vir::Node,
}

/// The output of a compile-time evaluated expression.
///
/// Type definition handles will be `None` wherever type inference fails.
#[derive(Debug)]
pub(crate) enum CEval {
	SelfPtr {
		typedef: Option<rti::Handle<TypeDef>>,
	},
	SuperPtr {
		typedef: Option<rti::Handle<TypeDef>>,
	},
	Type {
		handle: rti::Handle<TypeDef>,
	},
	/// Attempting to consume a value of this kind is a compiler error
	/// in non-native libraries.
	TypeDef {
		record: rti::Record,
	},
	Value {
		typedef: Option<rti::Handle<TypeDef>>,
		value: SmallVec<[vir::Immediate; 1]>,
	},
}

impl Clone for CEval {
	fn clone(&self) -> Self {
		match self {
			Self::SelfPtr { typedef } => Self::SelfPtr {
				typedef: typedef.clone(),
			},
			Self::SuperPtr { typedef } => Self::SuperPtr {
				typedef: typedef.clone(),
			},
			Self::Type { handle } => Self::Type {
				handle: handle.clone(),
			},
			Self::Value { typedef, value } => Self::Value {
				typedef: typedef.clone(),
				value: value.clone(),
			},
			Self::TypeDef { .. } => unreachable!(),
		}
	}
}

pub(crate) type CEvalVec = SmallVec<[CEval; 1]>;

#[derive(Debug, Clone)]
struct ScopeStack<'s> {
	compiler: &'s Compiler,
	namespace: &'s Scope,
	scopes: Scope,
}

impl ScopeStack<'_> {
	#[must_use]
	fn lookup(&self, nsname: NsName) -> Option<&Symbol> {
		todo!()
	}
}

/// Cache handles to types which will be commonly referenced to minimize lookups.
#[derive(Debug)]
struct TypeCache {
	bool_t: rti::Handle<TypeDef>,
	int32_t: rti::Handle<TypeDef>,
	uint32_t: rti::Handle<TypeDef>,
	int64_t: rti::Handle<TypeDef>,
	uint64_t: rti::Handle<TypeDef>,
	float32_t: rti::Handle<TypeDef>,
	float64_t: rti::Handle<TypeDef>,
	string_t: rti::Handle<TypeDef>,
	iname_t: rti::Handle<TypeDef>,
}
