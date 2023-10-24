use std::{
	any::TypeId,
	hash::{Hash, Hasher},
	marker::PhantomData,
};

use rustc_hash::{FxHashMap, FxHasher};

use crate::{compile::JitModule, interop::Interop, rti};

/// Context for Lithica execution.
///
/// Fully re-entrant; Lith has no global state.
#[derive(Debug)]
pub struct Runtime {
	pub(crate) _function_rti: FxHashMap<String, rti::Store<rti::Function>>,
	pub(crate) _data_rti: FxHashMap<String, rti::Store<rti::DataObj>>,
	pub(crate) _type_rti: FxHashMap<String, rti::Store<rti::Rtti>>,
	/// Left untouched by the runtime; just needs to be here so that its
	/// memory does not get freed until it has no more users.
	#[allow(unused)]
	pub(crate) module: JitModule,
	pub(crate) user_ctx_t: TypeId,
}

impl Runtime {
	/// `U` must be the same type as the one passed to [`crate::finalize`]
	/// or else `None` will be returned.
	pub fn downcast<'f, U: 'static, F: Interop<U>>(
		&self,
		function: &'f rti::Function,
	) -> Option<rti::TFn<'f, U, F>> {
		let typeid = TypeId::of::<U>();

		if typeid != self.user_ctx_t {
			return None;
		}

		let mut hasher = FxHasher::default();

		F::PARAMS.hash(&mut hasher);
		F::RETURNS.hash(&mut hasher);

		if function.sig_hash == hasher.finish() {
			return Some(rti::TFn(function, PhantomData));
		}

		None
	}
}

/// A pointer to a structure of this type gets weaved through all Lith calls.
#[derive(Debug)]
#[repr(C)]
pub struct Context<U> {
	pub rt: *mut Runtime,
	pub user: *mut U,
}

/// Type-erased counterpart of [`Context`] for internal use.
#[derive(Debug)]
#[repr(C)]
pub(crate) struct InContext {
	pub(crate) rt: *mut Runtime,
	user: *mut (),
}
