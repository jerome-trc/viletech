//! The VZ2C backend.
//!
//! This is a not-necessarily-final part of the compiler which:
//! - translates VZScript ASTs to C source code;
//! - compiles that C source code to [MIR](mir);
//! - and lowers the MIR to JIT code, or functions that can be interpreted.

use std::{
	ffi::{c_char, c_int, c_void},
	io::Cursor,
	sync::Arc,
};

use parking_lot::RwLock;

use crate::{
	compile::{Compiler, Optimization},
	Project, Runtime,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirType {
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	F32,
	F64,
	Pointer,
	Block,
	RetBlock,
}

#[must_use]
pub fn compile(compiler: Compiler, opt: Optimization) -> (Project, Arc<RwLock<Runtime>>) {
	unsafe {
		assert_eq!(mir::MIR_API_VERSION, mir::_MIR_get_api_version());
		let ctx = mir::_MIR_init();
		mir::c2mir_init(ctx);
		let par = std::thread::available_parallelism().map_or(1, |i| i.get() as i32);
		mir::MIR_gen_init(ctx, par);

		for g in 0..par {
			mir::MIR_gen_set_optimize_level(ctx, g, opt as u32);
		}
	}

	unsafe extern "C" fn _getc_func(userd: *mut c_void) -> c_int {
		let buf: *mut Cursor<String> = userd.cast();
		let pos = (*buf).position();
		let len = (*buf).get_ref().len() as u64;

		if pos >= len {
			-1 // libc's EOF
		} else {
			let r = (*buf).get_ref().as_ptr().add(pos as usize);
			(*buf).set_position((*buf).position() + 1);
			*r as c_int
		}
	}

	unsafe extern "C" fn import_resolver(_: *const c_char) -> *mut c_void {
		// If VZS ends up allowing privileged scripts to load
		// dynamic C libraries, consume them here.
		std::ptr::null_mut()
	}

	let runtime = Runtime::new();
	let runtime_ptr = runtime.data_ptr();

	(todo!(), runtime)
}
