//! Details of Lithica's [Cranelift](cranelift)-based backend.

use std::mem::MaybeUninit;

use cranelift_jit::JITModule;

/// Newtype providing `Send` and `Sync` implementations around a [`JITModule`],
/// and ensure that JIT memory gets freed at the correct time.
#[derive(Debug)]
pub(crate) struct JitModule(MaybeUninit<JITModule>);

impl std::ops::Deref for JitModule {
	type Target = JITModule;

	fn deref(&self) -> &Self::Target {
		// SAFETY: The `JITModule` within only goes uninitialized when this type's
		// destructor is run, and this type's interface protects against any
		// other de-initialization.
		unsafe { self.0.assume_init_ref() }
	}
}

impl std::ops::DerefMut for JitModule {
	fn deref_mut(&mut self) -> &mut Self::Target {
		// SAFETY: Same as `Deref`.
		unsafe { self.0.assume_init_mut() }
	}
}

impl Drop for JitModule {
	fn drop(&mut self) {
		unsafe {
			let definitely_init = std::mem::replace(&mut self.0, MaybeUninit::uninit());
			definitely_init.assume_init().free_memory();
		}
	}
}

unsafe impl Send for JitModule {}
unsafe impl Sync for JitModule {}
