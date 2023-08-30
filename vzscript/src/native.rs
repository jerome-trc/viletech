//! The [`Native`] trait for providing backend representations of Rust types.

use std::sync::Arc;

use smallvec::{smallvec, SmallVec};
use util::rstring::RString;

use crate::back::MirType;

/// Provides a backend representation of a Rust type for interop purposes.
///
/// # Safety
///
/// The return value of `repr` must precisely match the memory layout of `Self`.
pub unsafe trait Native {
	fn repr() -> SmallVec<[MirType; 1]>;
}

unsafe impl Native for () {
	fn repr() -> SmallVec<[MirType; 1]> {
		smallvec![]
	}
}

unsafe impl Native for i8 {
	fn repr() -> SmallVec<[MirType; 1]> {
		[MirType::I8].into()
	}
}

unsafe impl Native for u8 {
	fn repr() -> SmallVec<[MirType; 1]> {
		[MirType::I8].into()
	}
}

unsafe impl Native for i16 {
	fn repr() -> SmallVec<[MirType; 1]> {
		[MirType::I16].into()
	}
}

unsafe impl Native for u16 {
	fn repr() -> SmallVec<[MirType; 1]> {
		[MirType::I16].into()
	}
}

unsafe impl Native for i32 {
	fn repr() -> SmallVec<[MirType; 1]> {
		[MirType::I32].into()
	}
}

unsafe impl Native for u32 {
	fn repr() -> SmallVec<[MirType; 1]> {
		[MirType::I32].into()
	}
}

unsafe impl Native for i64 {
	fn repr() -> SmallVec<[MirType; 1]> {
		[MirType::I64].into()
	}
}

unsafe impl Native for u64 {
	fn repr() -> SmallVec<[MirType; 1]> {
		[MirType::I64].into()
	}
}

unsafe impl<T> Native for *const T {
	fn repr() -> SmallVec<[MirType; 1]> {
		smallvec![MirType::Pointer]
	}
}

unsafe impl<T> Native for *mut T {
	fn repr() -> SmallVec<[MirType; 1]> {
		smallvec![MirType::Pointer]
	}
}

unsafe impl<T> Native for Arc<T> {
	fn repr() -> SmallVec<[MirType; 1]> {
		smallvec![MirType::Pointer]
	}
}

unsafe impl Native for RString {
	fn repr() -> SmallVec<[MirType; 1]> {
		smallvec![MirType::Pointer]
	}
}
