//! Implementations of the [`super::Native`] trait.

use cranelift::prelude::types;

use crate::types::AbiType;

use super::Native;

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

unsafe impl<T: Native> Native for std::cell::UnsafeCell<T> {
	const REPR: AbiType = T::REPR;
}

unsafe impl<T: Native> Native for std::cell::Cell<T> {
	const REPR: AbiType = T::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicBool {
	const REPR: AbiType = bool::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicI8 {
	const REPR: AbiType = i8::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicI16 {
	const REPR: AbiType = i16::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicI32 {
	const REPR: AbiType = i32::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicI64 {
	const REPR: AbiType = i64::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicU8 {
	const REPR: AbiType = u8::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicU16 {
	const REPR: AbiType = u16::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicU32 {
	const REPR: AbiType = u32::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicU64 {
	const REPR: AbiType = u64::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicIsize {
	const REPR: AbiType = isize::REPR;
}

unsafe impl Native for std::sync::atomic::AtomicUsize {
	const REPR: AbiType = usize::REPR;
}

unsafe impl<T: 'static + Sized> Native for std::sync::atomic::AtomicPtr<T> {
	const REPR: AbiType = isize::REPR;
}

unsafe impl<T: 'static + Sized> Native for std::rc::Rc<T> {
	const REPR: AbiType = isize::REPR;
}

unsafe impl<T: 'static + Sized> Native for std::sync::Arc<T> {
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

const _STATIC_ASSERT_RC_WIDTH: () = {
	assert!(std::mem::size_of::<std::rc::Rc<i32>>() == std::mem::size_of::<isize>());
	assert!(std::mem::size_of::<std::sync::Arc<i32>>() == std::mem::size_of::<isize>());
};
