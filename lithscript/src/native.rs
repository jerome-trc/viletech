//! The [`Native`] trait for providing backend representations of Rust types.

use std::sync::Arc;

use cranelift::prelude::types;
use smallvec::{smallvec, SmallVec};
use util::rstring::RString;

use crate::BackendType;

/// Provides a backend representation of a Rust type for interop purposes.
///
/// # Safety
///
/// The return value of `repr` must precisely match the memory layout of `Self`.
pub unsafe trait Native {
	fn repr(ptr_bits: u16) -> SmallVec<[BackendType; 1]>;
}

unsafe impl Native for () {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		smallvec![]
	}
}

unsafe impl Native for i8 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I8].into()
	}
}

unsafe impl Native for u8 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I8].into()
	}
}

unsafe impl Native for i16 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I16].into()
	}
}

unsafe impl Native for u16 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I16].into()
	}
}

unsafe impl Native for i32 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I32].into()
	}
}

unsafe impl Native for u32 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I32].into()
	}
}

unsafe impl Native for i64 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I64].into()
	}
}

unsafe impl Native for u64 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I64].into()
	}
}

unsafe impl Native for i128 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I128].into()
	}
}

unsafe impl Native for u128 {
	fn repr(_: u16) -> SmallVec<[BackendType; 1]> {
		[types::I128].into()
	}
}

unsafe impl<T> Native for *const T {
	fn repr(ptr_bits: u16) -> SmallVec<[BackendType; 1]> {
		[BackendType::int(ptr_bits).unwrap()].into()
	}
}

unsafe impl<T> Native for *mut T {
	fn repr(ptr_bits: u16) -> SmallVec<[BackendType; 1]> {
		[BackendType::int(ptr_bits).unwrap()].into()
	}
}

unsafe impl<T> Native for Arc<T> {
	fn repr(ptr_bits: u16) -> SmallVec<[BackendType; 1]> {
		[BackendType::int(ptr_bits).unwrap()].into()
	}
}

unsafe impl Native for RString {
	fn repr(ptr_bits: u16) -> SmallVec<[BackendType; 1]> {
		[BackendType::int(ptr_bits).unwrap()].into()
	}
}
