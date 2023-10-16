use parking_lot::Mutex;

use crate::{back::JitModule, data::SymPtr, filetree::FileIx, Compiler, ParseTree, Scope, ValVec};

/// The result of a [compile-time evaluated expression](ceval).
#[derive(Debug)]
#[must_use]
pub(crate) enum CEval {
	Err,
	Container(Scope),
	Type(SymPtr),
	Value(ValVec),
}

pub(crate) struct SemaContext<'c> {
	pub(crate) compiler: &'c Compiler,
	pub(crate) module: &'c Mutex<JitModule>,
	pub(crate) arena: &'c bumpalo::Bump,
	pub(crate) file_ix: FileIx,
	pub(crate) path: &'c str,
	pub(crate) ptree: &'c ParseTree,
	pub(crate) scope: &'c Scope,
}

impl std::ops::Deref for SemaContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}
