//! VZScript's [Cranelift](cranelift)-based backend.

use std::{
	ffi::{c_char, c_int, c_void},
	io::Cursor,
	mem::MaybeUninit,
	sync::{
		atomic::{self, AtomicBool},
		Arc,
	},
};

use cranelift::prelude::settings::OptLevel;
use cranelift_interpreter::interpreter::InterpreterState;
use cranelift_jit::{JITBuilder, JITModule};
use crossbeam::{queue::SegQueue, utils::Backoff};
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;

use crate::{
	compile::{Compiler, Scope, SymbolPtr},
	front::{Symbol, Undefined},
	interpreter::Interpreter,
	Project, Runtime,
};

pub type SsaType = cranelift::codegen::ir::Type;
pub type SsaValues = smallvec::SmallVec<[SsaType; 1]>;

pub fn finish(compiler: &mut Compiler, opt: OptLevel) {
	let par = (rayon::current_num_threads().max(1) - 1).max(1);
	let mut workers = Vec::with_capacity(par);
	// let queue = SegQueue::new();
	let done = AtomicBool::new(false);

	workers.resize_with(par, || {
		let o_lvl = match opt {
			OptLevel::None => "none",
			OptLevel::Speed => "speed",
			OptLevel::SpeedAndSize => "speed_and_size",
		};

		let mut builder = JITBuilder::with_flags(
			&[
				("use_colocated_libcalls", "false"),
				("is_pic", "false"),
				("opt_level", o_lvl),
			],
			cranelift_module::default_libcall_names(),
		)
		.expect("JIT module builder creation failed");

		// builder.symbol_lookup_fn(symbol_lookup_fn);

		// CodeGenWorker {
		// 	interpreter: Interpreter::new(InterpreterState::default()).with_fuel(Some(1_000_000)),
		// 	module: Arc::new(JitModule {
		// 		inner: MaybeUninit::new(Mutex::new(JITModule::new(builder))),
		// 	}),
		// 	queue: &queue,
		// 	symbols: &compiler.symbols,
		// 	done: &done,
		// }
	});

	// rayon::join(
	// 	|| {
	// 		for (&iname, &sym_ix) in &compiler.global {
	// 			let symptr = compiler.symbols.get(usize::from(sym_ix)).unwrap();
	// 			let guard = symptr.load();

	// 			if !guard.is_defined() {
	// 				queue.push(sym_ix);
	// 			}
	// 		}
	// 	},
	// 	|| {
	// 		workers.par_iter_mut().for_each(|worker| {
	// 			worker.run();
	// 		});
	// 	},
	// );
}

// fn walk_scope(compiler: &Compiler, queue: &SegQueue<SymbolIx>, scope: &Scope) {
// 	for (&iname, &sym_ix) in scope.iter() {
// 		let symptr = compiler.symbols.get(usize::from(sym_ix)).unwrap();
// 		let guard = symptr.load();

// 		if let Symbol::Undefined { scope: lock, .. } = guard.as_ref() {
// 			let sub_scope = lock.read();
// 			walk_scope(compiler, queue, &sub_scope);
// 			queue.push(sym_ix);
// 		} else if let Some(sub_scope) = guard.scope() {
// 			walk_scope(compiler, queue, &sub_scope);
// 		}
// 	}
// }

/// To wrap in an [`Arc`] so that JIT memory is freed properly.
#[derive(Debug)]
pub(crate) struct JitModule {
	inner: MaybeUninit<Mutex<JITModule>>,
}

unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			std::mem::replace(&mut self.inner, MaybeUninit::uninit())
				.assume_init()
				.into_inner()
				.free_memory();
		}
	}
}

// struct CodeGenWorker<'a> {
// 	interpreter: Interpreter<'static>,
// 	module: Arc<JitModule>,
// 	queue: &'a SegQueue<SymbolIx>,
// 	symbols: &'a Slab<SymbolPtr>,
// 	done: &'a AtomicBool,
// }

// impl CodeGenWorker<'_> {
// 	fn run(&mut self) {
// 		let backoff = Backoff::new();

// 		while !self.done.load(atomic::Ordering::Acquire) {
// 			while let Some(sym_ix) = self.queue.pop() {
// 				let symptr = self.symbols.get(usize::from(sym_ix)).unwrap();
// 				symptr.rcu(|undef| self.define(undef));
// 			}

// 			backoff.snooze();
// 		}
// 	}

// 	fn define(&self, undef: &Arc<Symbol>) -> Arc<Symbol> {
// 		let Symbol::Undefined { kind, scope, .. } = undef.as_ref() else {
// 			unreachable!()
// 		};

// 		let mut guard = scope.write();
// 		let mut scope: &mut Scope = &mut guard;
// 		let scope = std::mem::take(scope);

// 		match kind {
// 			UndefKind::Class => todo!(),
// 			UndefKind::Enum => todo!(),
// 			UndefKind::FlagDef => todo!(),
// 			UndefKind::Function => todo!(),
// 			UndefKind::Property => todo!(),
// 			UndefKind::Struct => todo!(),
// 			UndefKind::Union => todo!(),
// 			UndefKind::Value { comptime, mutable } => todo!(),
// 		}

// 		Arc::new(todo!())
// 	}
// }
