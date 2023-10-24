//! Runtime function information.

use std::marker::PhantomData;

use cranelift_module::FuncId;

use crate::{
	interop::{self, Interop, Ret2},
	runtime::{self, Runtime},
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
}

/// A strongly-typed reference to a [JIT function pointer](Function).
#[derive(Debug)]
pub struct TFn<'f, U: 'static, F: Interop<U>>(
	pub(crate) &'f Function,
	#[allow(clippy::type_complexity)] pub(crate) PhantomData<(F, fn(*mut U))>,
);

impl<U: 'static> TFn<'_, U, fn(*mut runtime::Context<U>)> {
	#[allow(clippy::result_unit_err)]
	pub fn call(&self, rt: &mut Runtime, userdata: &mut U) -> Result<(), ()> {
		let mut ctx = runtime::Context {
			rt: std::ptr::addr_of_mut!(*rt),
			user: std::ptr::addr_of_mut!(*userdata).cast(),
		};

		let j = cee_scape::call_with_sigsetjmp(false, |_| unsafe {
			let func = *self.0.ptr.cast::<fn(*mut runtime::Context<U>)>();

			func(std::ptr::addr_of_mut!(ctx));

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
			impl<U: 'static, $($param),+, $($ret),+> TFn<'_, U, fn(*mut runtime::Context<U>, $($param),+) -> $tie<$($ret),+>>
			where
				$($param: interop::Native),+,
				$tie<$($ret),+>: interop::Return,
				$($ret: interop::Native),+,
			{
				#[allow(clippy::result_unit_err)]
				pub fn call(
					&self,
					rt: &mut Runtime,
					userdata: &mut U,
					$($paramname: $param),+,
				) -> Result<$tie<$($ret),+>, ()> {
					let mut ctx = runtime::Context {
						rt: std::ptr::addr_of_mut!(*rt),
						user: std::ptr::addr_of_mut!(*userdata).cast(),
					};

					let mut ret = unsafe { std::mem::zeroed() };

					let j = cee_scape::call_with_sigsetjmp(false, |_| unsafe {
						let func = *self.0.ptr.cast::<fn(*mut runtime::Context<U>, $($param),+) -> $tie<$($ret),+>>();

						ret = func(std::ptr::addr_of_mut!(ctx), $($paramname),+);

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
pub struct TFnHandle<U: 'static, F: Interop<U>>(
	pub(crate) Handle<Function>,
	#[allow(clippy::type_complexity)] pub(crate) PhantomData<(F, fn(*mut U))>,
);
