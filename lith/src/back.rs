//! Details of Lithica's [Cranelift](cranelift)-based backend.

use std::mem::MaybeUninit;

use cranelift_jit::JITModule;
use parking_lot::Mutex;

/// Newtype providing `Send` and `Sync` implementations around a [`JITModule`],
/// and ensure that JIT memory gets freed at the correct time.
#[derive(Debug)]
pub(crate) struct JitModule(MaybeUninit<Mutex<JITModule>>);

impl std::ops::Deref for JitModule {
	type Target = Mutex<JITModule>;

	fn deref(&self) -> &Self::Target {
		// SAFETY: The `JITModule` within only goes uninitialized when this type's
		// destructor is run, and this type's interface protects against any
		// other de-initialization.
		unsafe { self.0.assume_init_ref() }
	}
}

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			let definitely_init = std::mem::replace(&mut self.0, MaybeUninit::uninit());
			let mutex = definitely_init.assume_init();
			let module = mutex.into_inner();
			module.free_memory();
		}
	}
}

unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}
