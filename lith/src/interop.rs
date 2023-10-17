//! The code that facilitates Rust/Lithica interoperability.

use std::{
	cell::{Cell, UnsafeCell},
	rc::Rc,
	sync::{
		atomic::{
			AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicPtr,
			AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
		},
		Arc,
	},
};

use cranelift::{
	codegen::ir::{ArgumentExtension, ArgumentPurpose},
	prelude::{types, AbiParam},
};

use crate::AbiType;

/// A pointer to a JIT function.
///
/// All implementors of this type are function pointers with only one return type,
/// since returning a stable-layout structure from a JIT function is always sound,
/// but passing an aggregate (struct, tuple, array) to one is never sound.
pub trait JitFn: 'static + Sized {
	const PARAMS: &'static [AbiParam];
	const RETURNS: &'static [AbiParam];
}

impl<RET> JitFn for fn() -> RET
where
	RET: Native,
{
	const PARAMS: &'static [AbiParam] = &[];

	const RETURNS: &'static [AbiParam] = &[AbiParam {
		value_type: RET::REPR,
		purpose: ArgumentPurpose::Normal,
		extension: ArgumentExtension::None,
	}];
}

macro_rules! impl_jitfn {
	($( $($param:ident),+ -> () );+) => {
		$(
			impl<$($param),+> JitFn for fn($($param),+) -> ()
			where
				$($param: Native),+,
			{
				const PARAMS: &'static [AbiParam] = &[
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
			impl<$($param),+, $($ret),+> JitFn for fn($($param),+) -> $tie<$($ret),+>
			where
				$($param: Native),+,
				$tie<$($ret),+>: Return,
				$($ret: Native),+,
			{
				const PARAMS: &'static [AbiParam] = &[
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

impl_jitfn! {
	AA -> ();
	AA, AB -> ();
	AA, AB, AC -> ()
}

impl_jitfn! {
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

unsafe impl Native for bool {
	const REPR: AbiType = types::I8;
}

unsafe impl Native for char {
	const REPR: AbiType = types::I32;
}

unsafe impl Native for i8 {
	const REPR: AbiType = types::I8;
}

unsafe impl Native for i16 {
	const REPR: AbiType = types::I16;
}

unsafe impl Native for i32 {
	const REPR: AbiType = types::I32;
}

unsafe impl Native for i64 {
	const REPR: AbiType = types::I64;
}

unsafe impl Native for i128 {
	const REPR: AbiType = types::I128;
}

unsafe impl Native for u8 {
	const REPR: AbiType = types::I8;
}

unsafe impl Native for u16 {
	const REPR: AbiType = types::I16;
}

unsafe impl Native for u32 {
	const REPR: AbiType = types::I32;
}

unsafe impl Native for u64 {
	const REPR: AbiType = types::I64;
}

unsafe impl Native for u128 {
	const REPR: AbiType = types::I128;
}

unsafe impl Native for isize {
	#[cfg(target_pointer_width = "32")]
	const REPR: AbiType = types::I32;
	#[cfg(target_pointer_width = "64")]
	const REPR: AbiType = types::I64;
}

unsafe impl Native for usize {
	const REPR: AbiType = isize::REPR;
}

unsafe impl Native for f32 {
	const REPR: AbiType = types::F32;
}

unsafe impl Native for f64 {
	const REPR: AbiType = types::F64;
}

unsafe impl<T: 'static + Sized> Native for *mut T {
	const REPR: AbiType = isize::REPR;
}

unsafe impl<T: 'static + Sized> Native for *const T {
	const REPR: AbiType = isize::REPR;
}

unsafe impl<T: 'static + Sized> Native for Box<T> {
	const REPR: AbiType = isize::REPR;
}

unsafe impl<T: Native> Native for UnsafeCell<T> {
	const REPR: AbiType = T::REPR;
}

unsafe impl<T: Native> Native for Cell<T> {
	const REPR: AbiType = T::REPR;
}

unsafe impl Native for AtomicBool {
	const REPR: AbiType = bool::REPR;
}

unsafe impl Native for AtomicI8 {
	const REPR: AbiType = i8::REPR;
}

unsafe impl Native for AtomicI16 {
	const REPR: AbiType = i16::REPR;
}

unsafe impl Native for AtomicI32 {
	const REPR: AbiType = i32::REPR;
}

unsafe impl Native for AtomicI64 {
	const REPR: AbiType = i64::REPR;
}

unsafe impl Native for AtomicU8 {
	const REPR: AbiType = u8::REPR;
}

unsafe impl Native for AtomicU16 {
	const REPR: AbiType = u16::REPR;
}

unsafe impl Native for AtomicU32 {
	const REPR: AbiType = u32::REPR;
}

unsafe impl Native for AtomicU64 {
	const REPR: AbiType = u64::REPR;
}

unsafe impl Native for AtomicIsize {
	const REPR: AbiType = isize::REPR;
}

unsafe impl Native for AtomicUsize {
	const REPR: AbiType = usize::REPR;
}

unsafe impl<T: 'static + Sized> Native for AtomicPtr<T> {
	const REPR: AbiType = isize::REPR;
}

unsafe impl<T: 'static + Sized> Native for Rc<T> {
	const REPR: AbiType = isize::REPR;
}

unsafe impl<T: 'static + Sized> Native for Arc<T> {
	const REPR: AbiType = isize::REPR;
}

#[cfg(target_arch = "x86_64")]
unsafe impl Native for core::arch::x86_64::__m128 {
	const REPR: AbiType = types::F32X4;
}

#[cfg(target_arch = "x86_64")]
unsafe impl Native for core::arch::x86_64::__m128d {
	const REPR: AbiType = types::F64X2;
}

#[cfg(target_arch = "x86_64")]
unsafe impl Native for core::arch::x86_64::__m256 {
	const REPR: AbiType = types::F32X8;
}

#[cfg(target_arch = "x86_64")]
unsafe impl Native for core::arch::x86_64::__m256d {
	const REPR: AbiType = types::F64X4;
}

/// See any implementation of [`JitFn`].
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

const _STATIC_ASSERT_RC_WIDTH: () = {
	assert!(std::mem::size_of::<Rc<i32>>() == std::mem::size_of::<isize>());
	assert!(std::mem::size_of::<Arc<i32>>() == std::mem::size_of::<isize>());
};
