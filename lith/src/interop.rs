//! The code that facilitates Rust/Lithica interoperability.

mod native;

use cranelift::{
	codegen::ir::{ArgumentExtension, ArgumentPurpose},
	prelude::AbiParam,
};

use crate::{runtime::Runtime, types::AbiType};

#[cfg(target_pointer_width = "64")]
pub(crate) const PTR_T: cranelift::codegen::ir::Type = cranelift::codegen::ir::types::I64;
#[cfg(target_pointer_width = "32")]
pub(crate) const PTR_T: cranelift::codegen::ir::Type = cranelift::codegen::ir::types::I32;

/// Trait for pointers to Lithica functions (JIT, native, or intrinsic).
///
/// All implementors of this type are function pointers with only one return type,
/// since returning a stable-layout structure from a JIT function is always sound,
/// but passing an aggregate (struct, tuple, array) to one is never sound.
pub trait Interop: 'static + Sized {
	const PARAMS: &'static [AbiParam];
	const RETURNS: &'static [AbiParam];
}

impl Interop for fn(*mut Runtime) {
	const PARAMS: &'static [AbiParam] = &[AbiParam {
		value_type: PTR_T,
		purpose: ArgumentPurpose::Normal,
		extension: ArgumentExtension::None,
	}];

	const RETURNS: &'static [AbiParam] = &[];
}

impl<RET> Interop for fn(*mut Runtime) -> RET
where
	RET: Native,
{
	const PARAMS: &'static [AbiParam] = &[AbiParam {
		value_type: PTR_T,
		purpose: ArgumentPurpose::Normal,
		extension: ArgumentExtension::None,
	}];

	const RETURNS: &'static [AbiParam] = &[AbiParam {
		value_type: RET::REPR,
		purpose: ArgumentPurpose::Normal,
		extension: ArgumentExtension::None,
	}];
}

macro_rules! impl_interop {
	($( $($param:ident),+ -> () );+) => {
		$(
			impl<$($param),+> Interop for fn(*mut Runtime, $($param),+) -> ()
			where
				$($param: Native),+,
			{
				const PARAMS: &'static [AbiParam] = &[
					AbiParam {
						value_type: PTR_T,
						purpose: ArgumentPurpose::Normal,
						extension: ArgumentExtension::None,
					},
					$(
						AbiParam {
							value_type: $param::REPR,
							purpose: ArgumentPurpose::Normal,
							extension: ArgumentExtension::None,
						}
					),+
				];

				const RETURNS: &'static [AbiParam] = &[];
			}
		)+
	};
	($( $($param:ident),+ -> $tie:ident<$($ret:ident),+> );+) => {
		$(
			impl<$($param),+, $($ret),+> Interop for fn(*mut Runtime, $($param),+) -> $tie<$($ret),+>
			where
				$($param: Native),+,
				$tie<$($ret),+>: Return,
				$($ret: Native),+,
			{
				const PARAMS: &'static [AbiParam] = &[
					AbiParam {
						value_type: PTR_T,
						purpose: ArgumentPurpose::Normal,
						extension: ArgumentExtension::None,
					},
					$(
						AbiParam {
							value_type: $param::REPR,
							purpose: ArgumentPurpose::Normal,
							extension: ArgumentExtension::None,
						}
					),+
				];

				const RETURNS: &'static [AbiParam] = &[
					$(
						AbiParam {
							value_type: $ret::REPR,
							purpose: ArgumentPurpose::Normal,
							extension: ArgumentExtension::None,
						}
					),+
				];
			}
		)+
	};
}

impl_interop! {
	AA -> ();
	AA, AB -> ();
	AA, AB, AC -> ()
}

impl_interop! {
	AA -> Ret2<RA, RB>;
	AA, AB -> Ret2<RA, RB>;
	AA, AB, AC -> Ret2<RA, RB>
} // (RAT) Why does Rust not have variadic generics again?

/// # Safety
///
/// This trait is unsafe to implement since using an incorrect [`AbiType`] for a
/// Rust type will throw the memory correctness of all generated Lithica JIT code
/// into question.
///
/// There should be no reason for you to implement this trait yourself, since only
/// scalar and vector types have a corresponding `AbiType`.
pub unsafe trait Native: 'static + Sized {
	const REPR: AbiType;
}

/// See any implementation of [`Interop`].
///
/// This is separate from [`Native`] and since a JIT function can return
/// multiple values in a stable-layout aggregate but cannot be passed any aggregates.
///
/// # Safety
///
/// `sig_hash`'s output must precisely corresponds to `Self`'s ABI representation.
/// Failure to do so will render all generated Lithica JIT code unsound.
pub unsafe trait Return: 'static + Sized {}

unsafe impl<T: Native> Return for T {}

unsafe impl<T: Native, const LEN: usize> Return for [T; LEN] {}

#[repr(C)]
pub struct Ret2<A, B>
where
	A: Native,
	B: Native,
{
	pub a: A,
	pub b: B,
}

unsafe impl<A, B> Return for Ret2<A, B>
where
	A: Native,
	B: Native,
{
}
