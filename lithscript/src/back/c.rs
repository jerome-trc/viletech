//! The LithC backend lowers C code (translated from Lith by [CMid]) to [MIR].
//!
//! [CMid]: crate::cmid
//! [MIR]: mir

use std::ffi::{c_char, c_int, c_void, CString};

use crate::{
	cmid,
	compile::{self, Compiler, Optimization},
	extend::CompExt,
};

#[derive(Debug)]
pub struct Backend {
	pub(crate) ctx: mir::MIR_context_t,
}

impl Backend {
	#[must_use]
	pub fn new(opt: Optimization) -> Self {
		unsafe {
			assert_eq!(mir::MIR_API_VERSION, mir::_MIR_get_api_version());
			let ctx = mir::_MIR_init();
			mir::c2mir_init(ctx);
			let par = std::thread::available_parallelism().map_or(1, |i| i.get() as i32);
			mir::MIR_gen_init(ctx, par);

			for g in 0..par {
				mir::MIR_gen_set_optimize_level(ctx, g, opt as u32);
			}

			Self { ctx }
		}
	}

	/// Will panic if [`Compiler::seal`] has not already been called.
	pub fn invoke<E: CompExt>(&mut self, compiler: &mut Compiler<E>, c_sources: Vec<cmid::Source>) {
		#[derive(Debug)]
		struct Buffer {
			pos: usize,
			code: *const c_char,
		}

		unsafe extern "C" fn getc_func(userd: *mut c_void) -> c_int {
			let buf: *mut Buffer = userd.cast();
			let p = (*buf).pos;
			let mut c = *(*buf).code.add(p);

			if c == 0 {
				c = -1; // libc's EOF
			} else {
				(*buf).pos += 1;
			}

			c as c_int
		}

		unsafe extern "C" fn import_resolver(_: *const c_char) -> *mut c_void {
			// If Lith winds up supporting dynamic C libraries, consume them here.
			std::ptr::null_mut()
		}

		assert_eq!(compiler.stage, compile::Stage::CodeGen);
		assert!(compiler.cur_lib == compiler.sources.len());
		assert!(!compiler.any_errors());

		let mut module_ix = 0;

		unsafe {
			let mut options = mir::c2mir_options {
				message_file: mir::stderr,
				debug_p: i32::from(false),
				verbose_p: i32::from(false),
				ignore_warnings_p: i32::from(false),
				no_prepro_p: i32::from(false),
				prepro_only_p: i32::from(false),
				syntax_only_p: i32::from(false),
				pedantic_p: i32::from(false),
				asm_p: i32::from(false),
				object_p: i32::from(false),
				module_num: module_ix,
				prepro_output_file: std::ptr::null_mut(),
				output_file_name: std::ptr::null_mut(),
				macro_commands_num: 0,
				include_dirs_num: 0,
				macro_commands: std::ptr::null_mut(),
				include_dirs: std::ptr::null_mut(),
			};

			for csrc in c_sources {
				let c_text = CString::new(csrc.text).unwrap();
				let c_name = CString::new(csrc.name).unwrap();

				let mut srcbuf = Buffer {
					pos: 0,
					code: c_text.as_ptr(),
				};

				mir::c2mir_compile(
					self.ctx,
					std::ptr::addr_of_mut!(options),
					Some(getc_func),
					std::ptr::addr_of_mut!(srcbuf).cast(),
					c_name.as_ptr(),
					std::ptr::null_mut(),
				);

				module_ix += 1;
				options.module_num = module_ix;
			}

			// TODO: Iterate over all compiled modules and load them.
			let _ = mir::MIR_get_module_list(self.ctx);

			// Load necessities that don't fall under the corelib, such as `memcpy`.

			self.load_ext(
				"__memcpy__".to_string(),
				core::ptr::copy_nonoverlapping::<c_void> as *mut c_void,
			);

			mir::MIR_link(
				self.ctx,
				Some(mir::MIR_set_parallel_gen_interface),
				Some(import_resolver),
			);
		}
	}

	unsafe fn load_ext(&mut self, name: String, addr: *mut c_void) {
		let c_name = CString::new(name).unwrap();
		mir::MIR_load_external(self.ctx, c_name.as_ptr(), addr);
	}

	#[must_use]
	#[cfg(any())]
	pub fn find_module(&self, name: &str) -> Option<Module> {
		unsafe {
			let list = mir::MIR_get_module_list(backend.ctx);
			let mut walker = (*list).head;

			loop {
				if walker.is_null() {
					break;
				}

				let cname = CStr::from_ptr((*walker).name);

				if cname.to_str().is_ok_and(|n| n == name) {
					return Some(Module {
						backend: self.backend.clone(),
						inner: walker,
					});
				}

				walker = (*walker).module_link.next;
			}
		}

		None
	}
}

impl Drop for Backend {
	fn drop(&mut self) {
		unsafe {
			mir::MIR_gen_finish(self.ctx);
			mir::c2mir_finish(self.ctx);
			mir::MIR_finish(self.ctx);
		}
	}
}

unsafe impl Send for Backend {}
