use std::sync::atomic::{self, AtomicU32};

use cranelift::{
	codegen::{data_value::DataValue, ir::SourceLoc},
	prelude::{FunctionBuilderContext, Signature},
};
use parking_lot::Mutex;
use util::pushvec::PushVec;

use crate::{
	back::JitModule,
	data::SymPtr,
	filetree::{self, FileIx},
	types::Scope,
	Compiler, ParseTree,
};

/// The result of a [compile-time evaluated expression](ceval).
#[derive(Debug)]
#[must_use]
pub(crate) enum CEval {
	Err,
	Container(Scope),
	Type(SymPtr),
	Value(PushVec<DataValue>),
}

#[derive(Clone, Copy)]
pub(crate) struct SemaContext<'c> {
	pub(crate) tctx: ThreadContext<'c>,
	pub(crate) file_ix: FileIx,
	pub(crate) path: &'c str,
	pub(crate) ptree: &'c ParseTree,
}

#[derive(Clone, Copy)]
pub(crate) struct ThreadContext<'c> {
	pub(crate) thread_ix: usize,
	pub(crate) compiler: &'c Compiler,
	pub(crate) module: &'c Mutex<JitModule>,
	pub(crate) fctxs: &'c Vec<Mutex<FunctionBuilderContext>>,
	pub(crate) cctxs: &'c Vec<Mutex<cranelift::codegen::Context>>,
	pub(crate) base_sig: &'c Signature,
	pub(crate) next_src_loc: &'c AtomicU32,
}

impl std::ops::Deref for ThreadContext<'_> {
	type Target = Compiler;

	fn deref(&self) -> &Self::Target {
		self.compiler
	}
}

impl<'c> std::ops::Deref for SemaContext<'c> {
	type Target = ThreadContext<'c>;

	fn deref(&self) -> &Self::Target {
		&self.tctx
	}
}

impl<'c> SemaContext<'c> {
	#[must_use]
	fn fetch_next_src_loc(&self) -> SourceLoc {
		SourceLoc::new(self.next_src_loc.fetch_add(1, atomic::Ordering::SeqCst))
	}

	#[must_use]
	fn switch_file(&'c self, file_ix: FileIx) -> Self {
		let filetree::Node::File { ptree, path } = &self.ftree.graph[file_ix] else {
			unreachable!()
		};

		Self {
			tctx: self.tctx,
			file_ix,
			path: path.as_str(),
			ptree,
		}
	}
}
