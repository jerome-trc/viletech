//! The [`Native`] trait for providing backend representations of Rust types.

use std::sync::Arc;

use cranelift::prelude::types;
use util::rstring::RString;

use crate::back::AbiType;

#[cfg(target_pointer_width = "32")]
pub const POINTER_T: AbiType = types::I32;
#[cfg(target_pointer_width = "64")]
pub const POINTER_T: AbiType = types::I64;

/// Provides a backend representation of a Rust type for interop purposes.
///
/// # Safety
///
/// The value of `REPR` must precisely match the memory layout of `Self`.
pub unsafe trait Native {
	const REPR: &'static [AbiType];
}

unsafe impl Native for () {
	const REPR: &'static [AbiType] = &[];
}

unsafe impl Native for i8 {
	const REPR: &'static [AbiType] = &[types::I8];
}

unsafe impl Native for u8 {
	const REPR: &'static [AbiType] = &[types::I8];
}

unsafe impl Native for i16 {
	const REPR: &'static [AbiType] = &[types::I16];
}

unsafe impl Native for u16 {
	const REPR: &'static [AbiType] = &[types::I16];
}

unsafe impl Native for i32 {
	const REPR: &'static [AbiType] = &[types::I32];
}

unsafe impl Native for u32 {
	const REPR: &'static [AbiType] = &[types::I32];
}

unsafe impl Native for i64 {
	const REPR: &'static [AbiType] = &[types::I64];
}

unsafe impl Native for u64 {
	const REPR: &'static [AbiType] = &[types::I64];
}

unsafe impl<T> Native for *const T {
	const REPR: &'static [AbiType] = &[POINTER_T];
}

unsafe impl<T> Native for *mut T {
	const REPR: &'static [AbiType] = &[POINTER_T];
}

unsafe impl<T> Native for Arc<T> {
	const REPR: &'static [AbiType] = &[POINTER_T];
}

unsafe impl Native for RString {
	const REPR: &'static [AbiType] = &[POINTER_T];
}
