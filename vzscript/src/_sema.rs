//! The semantic "mid-section" of the compiler.
//!
//! Name resolution, semantic checks, and compile-time evaluation.

mod vzs;
mod zs;

use std::{pin::Pin, sync::Arc};

use crossbeam::utils::Backoff;
use rayon::prelude::*;
use stable_deref_trait::StableDeref;

use crate::{
	compile::{
		self,
		intern::{NsName, SymbolIx},
		symbol::{Location, Symbol},
		Compiler, Scope,
	},
	rti,
	tsys::TypeDef,
	vir,
};

pub fn sema(compiler: &mut Compiler) {
	assert_eq!(compiler.stage, compile::Stage::Semantic);
	assert!(!compiler.failed);

	for (i, namespace) in compiler.namespaces.iter().enumerate() {
		namespace.par_iter().for_each(|(_, &sym_ix)| {
			let symptr = compiler.symbol(sym_ix);

			let (is_undefined, is_from_zscript, location) = {
				let g = symptr.load();

				let Some(location) = g.location else {
					// This symbol was defined by the compiler itself.
					// We can early out entirely here.
					return;
				};

				(g.is_undefined(), g.zscript, location)
			};

			if !is_undefined {
				// Defined by the compiler itself, or pending a definition.
				return;
			}

			let path = compiler.resolve_path(location);

			let undef = symptr.swap(Arc::new(Symbol {
				location: Some(location),
				source: None,
				def: Definition::Pending,
				zscript: is_from_zscript,
			}));

			let maybe_defined = if is_from_zscript {
				let ctx = SemaContext {
					compiler,
					location,
					path,
					namespace_ix: i,
				};

				zs::define(&ctx, undef)
			} else {
				let ctx = SemaContext {
					compiler,
					location,
					path,
					namespace_ix: i,
				};

				vzs::define(&ctx, undef)
			};

			let Some(defined) = maybe_defined else {
				debug_assert!({
					let g = symptr.load();
					g.is_defined()
				});

				return;
			};

			symptr.store(Arc::new(defined));
		});
	}

	compiler.stage = compile::Stage::CodeGen;

	if compiler.any_errors() {
		compiler.failed = true;
	}
}

#[derive(Debug)]
pub(self) struct SemaContext<'c> {
	pub(self) compiler: &'c Compiler,
	pub(self) location: Location,
	pub(self) path: &'c str,
	pub(self) namespace_ix: usize,
}

impl std::ops::Deref for SemaContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}

impl<'c> SemaContext<'c> {
	#[must_use]
	pub(self) fn clone_with<'o: 'c>(&'c self, location: Location) -> Self {
		Self {
			location,
			path: self.resolve_path(location),
			..*self
		}
	}

	#[must_use]
	pub(self) fn scope_stack(&self, zscript: bool) -> ScopeStack {
		let mut scopes = ScopeStack(vec![], self.compiler);
		scopes.push(StackedScope::Unguarded(&self.compiler.globals));

		if zscript {
			for (ii, ns) in self.namespaces.iter().enumerate() {
				if ii == self.namespace_ix {
					continue;
				}

				scopes.push(StackedScope::Unguarded(ns));
			}
		}

		scopes.push(StackedScope::Unguarded(&self.namespaces[self.namespace_ix]));

		scopes
	}

	#[must_use]
	pub(self) fn global_backlookup(&self, nsname: NsName) -> Option<SymbolIx> {
		self.namespaces[..self.namespace_ix]
			.iter()
			.rev()
			.find_map(|ns| ns.get(&nsname).copied())
	}

	#[must_use]
	pub(self) fn error_symbol(&self, zscript: bool) -> Symbol {
		Symbol {
			location: Some(self.location),
			source: None,
			def: Definition::Error,
			zscript,
		}
	}
}

#[must_use]
pub(self) fn swap_for_pending(
	symptr: &SymbolPtr,
	location: Location,
	zscript: bool,
) -> Arc<Symbol> {
	symptr.swap(Arc::new(Symbol {
		location: Some(location),
		source: None,
		def: Definition::Pending,
		zscript,
	}))
}

pub(self) struct ConstEval {
	/// `None` if the type cannot be known, such as when compile-time-evaluating
	/// a null pointer literal.
	pub(self) typedef: Option<rti::Handle<TypeDef>>,
	pub(self) ir: vir::Node,
}

#[derive(Debug)]
pub(self) enum StackedScope<'s> {
	Unguarded(&'s Scope),
	Guarded(selfie::Selfie<'s, SymbolGuard, ScopeRefProxy>),
}

impl StackedScope<'_> {
	#[must_use]
	pub(self) fn guarded(guard: arc_swap::Guard<Arc<Symbol>>) -> Self {
		let sguard = SymbolGuard(guard);
		let scope = sguard.scope().unwrap();

		Self::Guarded(selfie::Selfie::new(Pin::new(sguard), |h| {
			ScopeRef(h.scope().unwrap())
		}))
	}
}

impl std::ops::Deref for StackedScope<'_> {
	type Target = Scope;

	fn deref(&self) -> &Self::Target {
		match self {
			StackedScope::Unguarded(scope) => scope,
			StackedScope::Guarded(selfref) => selfref.referential().0,
		}
	}
}

#[derive(Debug)]
pub(self) struct SymbolGuard(arc_swap::Guard<Arc<Symbol>>);

impl std::ops::Deref for SymbolGuard {
	type Target = Arc<Symbol>;

	fn deref(&self) -> &Self::Target {
		std::ops::Deref::deref(&self.0)
	}
}

unsafe impl StableDeref for SymbolGuard {}

#[derive(Debug, Clone, Copy)]
pub(self) struct ScopeRef<'s>(&'s Scope);

#[derive(Debug, Clone, Copy)]
pub(self) struct ScopeRefProxy;

impl<'s> selfie::refs::RefType<'s> for ScopeRefProxy {
	type Ref = ScopeRef<'s>;
}

#[derive(Debug)]
pub(self) struct ScopeStack<'s>(Vec<StackedScope<'s>>, &'s Compiler);

impl<'s> std::ops::Deref for ScopeStack<'s> {
	type Target = Vec<StackedScope<'s>>;

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
	pub(crate) fn lookup(&self, nsname: NsName) -> Option<&SymbolPtr> {
		self.0
			.iter()
			.rev()
			.find_map(|scope| scope.get(&nsname).map(|&sym_ix| self.1.symbol(sym_ix)))
	}
}

pub(self) fn require(symptr: &SymbolPtr) {
	let guard = symptr.load();

	if guard.is_undefined() {
		todo!("define")
	}

	let backoff = Backoff::new();

	while guard.definition_pending() {
		backoff.snooze();
	}

	debug_assert!({
		let g = symptr.load();
		g.is_defined()
	});
}
