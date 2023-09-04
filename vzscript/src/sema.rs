//! The semantic "mid-section" of the compiler.
//!
//! Name resolution, semantic checks, and compile-time evaluation.

use crossbeam::utils::Backoff;
use rayon::prelude::*;
use util::rstring::RString;

use crate::{
	compile::{Compiler, Scope, SymbolPtr},
	front::{Symbol, Undefined},
	inctree::SourceKind,
	rti,
	tsys::TypeDef,
	vir,
	zname::ZName,
};

mod vzs;
mod zs;

pub(crate) fn sema(compiler: &Compiler) {
	assert!(!compiler.any_errors());

	for libsrc in &compiler.sources {
		libsrc.inctree.files.par_iter().for_each(|pfile| {
			let mut scopes = ScopeStack::default();

			scopes.push(StackedScope {
				inner: &compiler.globals,
				is_addendum: false,
			});

			let mut ctx = SemaContext {
				compiler,
				ipath: &compiler.intern_path(pfile.path()),
				scopes,
			};
		});
	}
}

#[derive(Debug)]
pub(self) struct SemaContext<'c> {
	pub(self) compiler: &'c Compiler,
	pub(self) ipath: &'c RString,
	pub(self) scopes: ScopeStack<'c>,
}

impl std::ops::Deref for SemaContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}

pub(self) struct ConstEval {
	/// `None` if the type cannot be known, such as when compile-time-evaluating
	/// a null pointer literal.
	pub(self) typedef: Option<rti::Handle<TypeDef>>,
	pub(self) ir: vir::Node,
}

#[derive(Debug)]
pub(self) struct StackedScope<'s> {
	pub(self) inner: &'s Scope,
	pub(self) is_addendum: bool,
}

#[derive(Debug, Default)]
pub(self) struct ScopeStack<'s>(Vec<StackedScope<'s>>);

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
	pub(crate) fn lookup(&self, iname: &ZName) -> Option<&SymbolPtr> {
		self.0.iter().rev().find_map(|scope| scope.inner.get(iname))
	}

	/// Used for classes with parent scopes, for example.
	/// Pops once unconditionally (e.g. a class itself) and then keeps popping
	/// until the last element is not an addendum.
	pub(crate) fn pop_with_addenda(&mut self) {
		let _ = self.0.pop().unwrap();

		while self.0.last().is_some_and(|s| s.is_addendum) {
			let _ = self.0.pop().unwrap();
		}
	}
}
