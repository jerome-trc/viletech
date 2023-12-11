//! See [`semantic_check`].

use std::cell::RefCell;

use cranelift::{
	codegen::ir::SourceLoc,
	prelude::{FunctionBuilderContext, Signature},
};
use cranelift_module::Module;
use doomfront::rowan::{ast::AstNode, TextSize};
use parking_lot::Mutex;

use crate::{
	ast,
	compile::{self, module::JitModule},
	filetree::{self, FileIx},
	issue::{self, Issue},
	types::{Scope, SymPtr, TypeOPtr, TypePtr},
	Compiler, ParseTree, ValVec,
};

use super::{
	ceval, func,
	sym::{self, ConstInit, Location, SymDatum, SymbolId},
	tsys::TypeDef,
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
	let mut lctxs = vec![];

	for _ in 0..rayon::current_num_threads() {
		lctxs.push(Mutex::new(LowerContext {
			fctx: RefCell::new(FunctionBuilderContext::new()),
			cctx: RefCell::new(module.make_context()),
			sig: RefCell::new(module.make_signature()),
		}));
	}

	let ptr_t = module.isa().pointer_type();
	let module = Mutex::new(module);

	// First, define and cache primitive types.

	{
		let folder_corelib = compiler
			.ftree
			.find_child(compiler.ftree.root(), "lith")
			.unwrap();
		let file_prim = compiler
			.ftree
			.find_child(folder_corelib, "primitive")
			.unwrap();

		let ftn = &compiler.ftree.graph[file_prim];

		let filetree::Node::File { ptree, path } = ftn else {
			return;
		};

		let thread_ix = 0;
		let arena = compiler.arenas[thread_ix].lock();

		let ctx = SemaContext {
			tctx: ThreadContext {
				thread_ix,
				compiler,
				arena: &arena,
				module: &module,
				lctxs: &lctxs,
				ptr_t,
			},
			file_ix: file_prim,
			path: path.as_str(),
			ptree,
		};

		semantic_check_container(&ctx);
	}

	// Finally, start initializing container values and defining crucial functions.

	if compiler.any_errors() {
		compiler.failed = true;
	} else {
		compiler.module = Some(module.into_inner());
		compiler.stage = compile::Stage::CodeGen;
	}
}

fn semantic_check_container(ctx: &SemaContext) {
	let cursor = ctx.ptree.cursor();

	for item in cursor.children().filter_map(ast::Item::cast) {
		let sym_id = SymbolId::new(Location {
			file_ix: ctx.file_ix,
			span: item.syntax().text_range(),
		});

		match item {
			ast::Item::Function(fndecl) => {
				for anno in fndecl.annotations() {
					let ast::AnnotationName::Unscoped(ident) = anno.name().unwrap() else {
						continue;
					};

					if ident.text() != "crucial" {
						continue;
					}

					check_function(ctx, fndecl, sym_id);
					break;
				}
			}
			ast::Item::SymConst(symconst) => {
				check_symconst(ctx, symconst, sym_id);
			}
		}
	}
}

fn check_function(ctx: &SemaContext, ast: ast::FunctionDecl, sym_id: SymbolId) {
	if let Some(ret_t) = ast.return_type() {
		ctx.raise(
			Issue::new(
				ctx.path,
				ret_t.syntax().text_range(),
				issue::Level::Error(issue::Error::Unimplemented),
			)
			.with_message_static("crucial functions do not support return values yet"),
		);

		return;
	}

	if let Some(param0) = ast.params().unwrap().iter().next() {
		ctx.raise(
			Issue::new(
				ctx.path,
				param0.syntax().text_range(),
				issue::Level::Error(issue::Error::Unimplemented),
			)
			.with_message_static("crucial functions do not support parameters yet"),
		);

		return;
	}

	let mono_sig = MonoSig {
		params: vec![],
		ret_t: ctx.sym_cache.void_t.clone().into(),
	};

	let kvp = ctx.symbols.get(&sym_id).unwrap();

	let SymDatum::Function(d_fn) = &kvp.datum else {
		unreachable!()
	};

	let file_loc = Location::full_file(ctx.file_ix);
	let ctr_env = ctx.scopes.get(&file_loc).unwrap();

	let _ = func::eager_define(ctx, &ctr_env, kvp.value(), d_fn, ast, mono_sig);
}

fn check_symconst(ctx: &SemaContext, ast: ast::SymConst, sym_id: SymbolId) {
	let kvp = ctx.symbols.get(&sym_id).unwrap();

	let SymDatum::SymConst(d_const) = &kvp.datum else {
		unreachable!()
	};

	let file_loc = Location::full_file(ctx.file_ix);
	let ctr_env = ctx.scopes.get(&file_loc).unwrap();
	let init_ast = ast.expr().unwrap();
	let init_span = init_ast.syntax().text_range();
	let init_eval = ceval::expr(ctx, 0, ctr_env.value(), init_ast);

	match &d_const.init {
		ConstInit::Type(init_ty) => match init_eval {
			CEval::Err => {}
			CEval::Function(_) => {
				ctx.raise(
					Issue::new(
						ctx.path,
						init_span,
						issue::Level::Error(issue::Error::AssignTypeMismatch),
					)
					.with_message_static(
						"cannot assign a function to a symbolic constant of type `type_t`",
					),
				);
			}
			CEval::Type(t) => {
				init_ty.store(t);
			}
			CEval::Value(_) => {
				ctx.raise(
					Issue::new(
						ctx.path,
						init_span,
						issue::Level::Error(issue::Error::AssignTypeMismatch),
					)
					.with_message_static(
						"cannot assign values to a symbolic constant of type `type_t`",
					),
				);
			}
		},
		ConstInit::Value(init_val) => match init_eval {
			CEval::Err => {}
			CEval::Function(_) => {
				ctx.raise(
					Issue::new(
						ctx.path,
						init_span,
						issue::Level::Error(issue::Error::Unimplemented),
					)
					.with_message_static("cannot assign a function to a symbolic constant")
					.with_note_static("coercion to a function pointer is not yet implemented"),
				);

				// TODO: coerce to function pointer?
			}
			CEval::Type(_) => {
				ctx.raise(
					Issue::new(
						ctx.path,
						init_span,
						issue::Level::Error(issue::Error::AssignTypeMismatch),
					)
					.with_message_static(
						"can only assign a type to a symbolic constant of type `type_t`",
					),
				);
			}
			CEval::Value(cevalue) => {
				let sym::TypeSpec::Normal(tspec) = &d_const.tspec else {
					unreachable!()
				};

				if *tspec != cevalue.ftype {
					// TODO: raise an error.
					return;
				}

				for val in cevalue.data {
					let _ = init_val.push(val);
				}
			}
		},
	}
}

/// A key into [`Compiler::memo`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MonoKey {
	pub(crate) func: SymPtr,
	pub(crate) sig: MonoSig,
}

/// "Monomorphized signature".
/// The result of
///
/// - replacing `any_t` parameter types with real types, and
/// - eliminating `type_t` parameters
///
/// at the call site.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MonoSig {
	pub(crate) params: Vec<TypePtr>,
	pub(crate) ret_t: TypePtr,
}

/// The result of a [**c**ompile-time **eval**uated expression](ceval).
#[derive(Debug)]
#[must_use]
pub enum CEval {
	Err,
	Function(SymPtr),
	Type(TypePtr),
	Value(CeValue),
}

/// "**C**onstant-**e**valuated **value**". Cranelift [`DataValue`]s with an attached type.
///
/// [`DataValue`]: cranelift::codegen::data_value::DataValue
#[derive(Debug)]
pub struct CeValue {
	pub(crate) data: ValVec,
	pub(crate) ftype: TypePtr,
}

#[derive(Clone, Copy)]
pub struct SemaContext<'c> {
	pub(crate) tctx: ThreadContext<'c>,
	pub(crate) file_ix: FileIx,
	pub(crate) path: &'c str,
	pub(crate) ptree: &'c ParseTree,
}

#[derive(Clone, Copy)]
pub struct ThreadContext<'c> {
	pub(crate) thread_ix: usize,
	pub(crate) compiler: &'c Compiler,
	pub(crate) arena: &'c bumpalo::Bump,
	pub(crate) module: &'c Mutex<JitModule>,
	pub(crate) lctxs: &'c Vec<Mutex<LowerContext>>,
	pub(crate) ptr_t: cranelift::codegen::ir::Type,
}

pub(crate) struct LowerContext {
	pub(crate) fctx: RefCell<FunctionBuilderContext>,
	pub(crate) cctx: RefCell<cranelift::codegen::Context>,
	pub(crate) sig: RefCell<Signature>,
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
			return TypePtr::from(ptr.key());
		}

		let ptr = TypeOPtr::alloc(self.arena, typedef);
		let ret = TypePtr::from(&ptr);
		self.types.insert(ptr);
		ret
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
