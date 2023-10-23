use std::sync::atomic::{self, AtomicU32};

use cranelift::{
	codegen::ir::SourceLoc,
	prelude::{FunctionBuilderContext, Signature},
};
use cranelift_module::Module;
use parking_lot::Mutex;

use crate::{
	compile::{self, JitModule},
	filetree::{self, FileIx},
	tsys::TypeDef,
	types::{Scope, TypePtr},
	Compiler, ParseTree, ValVec,
};

/// The "semantic mid-section" between Lith's frontend and backend.
///
/// Here:
/// - types of function parameters and return types are computed
/// - types of symbolic constants and static variables are computed
/// - function bodies are checked and have IR generated
/// - symbolic constant and static variable initializers are evaluated
pub fn semantic_check(compiler: &mut Compiler) {
	assert!(!compiler.failed);
	assert_eq!(compiler.stage, compile::Stage::Sema);
	assert_eq!(compiler.arenas.len(), rayon::current_num_threads());

	let module = JitModule::new(compiler);
	let mut fctxs = vec![];
	let mut cctxs = vec![];

	for _ in 0..rayon::current_num_threads() {
		fctxs.push(Mutex::new(FunctionBuilderContext::new()));
		cctxs.push(Mutex::new(module.make_context()));
	}

	let base_sig = module.make_signature();
	let module = Mutex::new(module);

	// TODO

	if compiler.any_errors() {
		compiler.failed = true;
	} else {
		compiler.module = Some(module.into_inner());
		compiler.stage = compile::Stage::CodeGen;
	}
}

/// The result of a [compile-time evaluated expression](ceval).
#[derive(Debug)]
#[must_use]
pub(crate) enum CEval {
	Err,
	Container(Scope),
	Type(TypePtr),
	Value(ValVec),
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
	pub(crate) arena: &'c bumpalo::Bump,
	pub(crate) module: &'c Mutex<JitModule>,
	pub(crate) fctxs: &'c Vec<Mutex<FunctionBuilderContext>>,
	pub(crate) cctxs: &'c Vec<Mutex<cranelift::codegen::Context>>,
	pub(crate) base_sig: &'c Signature,
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
	pub(crate) fn intern_type(&self, typedef: TypeDef) -> TypePtr {
		if let Some(ptr) = self.types.get(&typedef) {
			return *ptr.key();
		}

		let ptr = TypePtr::alloc(self.arena, typedef);
		self.types.insert(ptr);
		ptr
	}

	#[must_use]
	pub(crate) fn make_srcloc(&self, offs: TextSize) -> SourceLoc {
		// 11 high bits are reserved for 2048 files.
		// 21 low bits are reserved for 2,097,152 bytes per file.

		// For reference, GZDoom b48caddb9 has 700 translation units
		// and 784 header files (third-party code included).
		// Its largest header (vk_mem_alloc.h) has 731,933 bytes in it.

		let mut srcloc = 0;
		srcloc |= (self.file_ix.index() as u32).overflowing_shl(21).0;
		srcloc |= u32::from(offs) & 0x001FFFFF;
		SourceLoc::new(srcloc)
	}

	#[must_use]
	pub(crate) fn switch_file(&'c self, file_ix: FileIx) -> Self {
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

#[cfg(test)]
#[test]
fn srcloc_roundtrip() {
	let file_ix: u32 = 700 + 784;
	let offs: u32 = 731_933;

	let mut srcloc = 0;
	srcloc |= file_ix.overflowing_shl(21).0;
	srcloc |= offs & 0x001FFFFF;

	let file_ix = (srcloc & 0xFFE00000) >> 21;
	let offs = srcloc & 0x001FFFFF;

	assert_eq!((file_ix, offs), (700 + 784, 731_933));
}
