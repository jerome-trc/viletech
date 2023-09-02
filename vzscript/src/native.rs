//! The [`Native`] trait for providing backend representations of Rust types.

use std::sync::Arc;

use cranelift::prelude::types;
use smallvec::{smallvec, SmallVec};
use util::rstring::RString;

use crate::back::{SsaType, SsaValues};

#[cfg(target_pointer_width = "32")]
pub const POINTER_T: SsaType = types::I32;
#[cfg(target_pointer_width = "64")]
pub const POINTER_T: SsaType = types::I64;

/// Provides a backend representation of a Rust type for interop purposes.
///
/// # Safety
///
/// The return value of `repr` must precisely match the memory layout of `Self`.
pub unsafe trait Native {
	fn repr() -> SsaValues;
}

unsafe impl Native for () {
	fn repr() -> SsaValues {
		smallvec![]
	}
}

unsafe impl Native for i8 {
	fn repr() -> SsaValues {
		[types::I8].into()
	}
}

unsafe impl Native for u8 {
	fn repr() -> SsaValues {
		[types::I8].into()
	}
}

unsafe impl Native for i16 {
	fn repr() -> SsaValues {
		[types::I16].into()
	}
}

unsafe impl Native for u16 {
	fn repr() -> SsaValues {
		[types::I16].into()
	}
}

unsafe impl Native for i32 {
	fn repr() -> SsaValues {
		[types::I32].into()
	}
}

unsafe impl Native for u32 {
	fn repr() -> SsaValues {
		[types::I32].into()
	}
}

unsafe impl Native for i64 {
	fn repr() -> SsaValues {
		[types::I64].into()
	}
}

unsafe impl Native for u64 {
	fn repr() -> SsaValues {
		[types::I64].into()
	}
}

unsafe impl<T> Native for *const T {
	fn repr() -> SsaValues {
		smallvec![POINTER_T]
	}
}

unsafe impl<T> Native for *mut T {
	fn repr() -> SsaValues {
		smallvec![POINTER_T]
	}
}

unsafe impl<T> Native for Arc<T> {
	fn repr() -> SsaValues {
		smallvec![POINTER_T]
	}
}

unsafe impl Native for RString {
	fn repr() -> SsaValues {
		smallvec![POINTER_T]
	}
}
