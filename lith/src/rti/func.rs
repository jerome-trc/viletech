//! Runtime function information.

use std::{
	hash::{Hash, Hasher},
	marker::PhantomData,
};

use cranelift_module::FuncId;
use rustc_hash::FxHasher;

use crate::{
	interop::{self, Interop, Ret2},
	runtime::Runtime,
};

use super::Handle;

#[derive(Debug)]
pub struct Function {
	pub(crate) ptr: *const (),
	pub(crate) id: FuncId,
	pub(crate) sig_hash: u64,
}

impl Function {
	#[must_use]
	pub fn id(&self) -> FuncId {
		self.id
	}

	#[must_use]
	pub fn downcast<F: Interop>(&self) -> Option<TFn<F>> {
		let mut hasher = FxHasher::default();
		F::PARAMS.hash(&mut hasher);
		F::RETURNS.hash(&mut hasher);
		(self.sig_hash == hasher.finish()).then_some(TFn(self, PhantomData))
	}
}

/// A strongly-typed reference to a [JIT function pointer](Function).
#[derive(Debug)]
pub struct TFn<'f, F: Interop>(pub(crate) &'f Function, PhantomData<F>);

impl TFn<'_, fn(*mut Runtime)> {
	#[allow(clippy::result_unit_err)]
	pub fn call(&self, rt: &mut Runtime) -> Result<(), ()> {
		let j = cee_scape::call_with_sigsetjmp(false, |_| unsafe {
			let func = *self.0.ptr.cast::<fn(*mut Runtime)>();

			func(std::ptr::addr_of_mut!(*rt));

			0
		});

		match j {
			0 => Ok(()),
			_ => Err(()),
		}
	}
}

macro_rules! tfn_impl {
	($( $($paramname:ident : $param:ident),+ -> $tie:ident<$($ret:ident),+> );+) => {
		$(
			impl<$($param),+, $($ret),+> TFn<'_, fn(*mut Runtime, $($param),+) -> $tie<$($ret),+>>
			where
				$($param: interop::Native),+,
				$tie<$($ret),+>: interop::Return,
				$($ret: interop::Native),+,
			{
				#[allow(clippy::result_unit_err)]
				pub fn call(
					&self,
					rt: &mut Runtime,
					$($paramname: $param),+,
				) -> Result<$tie<$($ret),+>, ()> {
					let mut ret = unsafe { std::mem::zeroed() };

					let j = cee_scape::call_with_sigsetjmp(false, |_| unsafe {
						let func = *self.0.ptr.cast::<fn(*mut Runtime, $($param),+) -> $tie<$($ret),+>>();

						ret = func(std::ptr::addr_of_mut!(*rt), $($paramname),+);

						0
					});

					match j {
						0 => Ok(ret),
						_ => Err(()),
					}
				}
			}
		)+
	};
}

tfn_impl! {
	aa: AA -> Ret2<RA, RB>;
	aa: AA, ab: AB -> Ret2<RA, RB>
}

/// A strongly-typed [handle](Handle) to a [JIT function pointer](Function).
#[derive(Debug)]
pub struct TFnHandle<F: Interop>(pub(crate) Handle<Function>, PhantomData<F>);
