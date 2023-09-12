//! The [`Native`] trait for providing backend representations of Rust types.

use std::sync::Arc;

use cranelift::prelude::types;
use smallvec::smallvec;
use util::rstring::RString;

use crate::back::{AbiType, AbiTypes};

#[cfg(target_pointer_width = "32")]
pub const POINTER_T: AbiType = types::I32;
#[cfg(target_pointer_width = "64")]
pub const POINTER_T: AbiType = types::I64;

/// Provides a backend representation of a Rust type for interop purposes.
///
/// # Safety
///
/// The return value of `repr` must precisely match the memory layout of `Self`.
pub unsafe trait Native {
	fn repr() -> AbiTypes;
}

unsafe impl Native for () {
	fn repr() -> AbiTypes {
		smallvec![]
	}
}

unsafe impl Native for i8 {
	fn repr() -> AbiTypes {
		[types::I8].into()
	}
}

unsafe impl Native for u8 {
	fn repr() -> AbiTypes {
		[types::I8].into()
	}
}

unsafe impl Native for i16 {
	fn repr() -> AbiTypes {
		[types::I16].into()
	}
}

unsafe impl Native for u16 {
	fn repr() -> AbiTypes {
		[types::I16].into()
	}
}

unsafe impl Native for i32 {
	fn repr() -> AbiTypes {
		[types::I32].into()
	}
}

unsafe impl Native for u32 {
	fn repr() -> AbiTypes {
		[types::I32].into()
	}
}

unsafe impl Native for i64 {
	fn repr() -> AbiTypes {
		[types::I64].into()
	}
}

unsafe impl Native for u64 {
	fn repr() -> AbiTypes {
		[types::I64].into()
	}
}

unsafe impl<T> Native for *const T {
	fn repr() -> AbiTypes {
		smallvec![POINTER_T]
	}
}

unsafe impl<T> Native for *mut T {
	fn repr() -> AbiTypes {
		smallvec![POINTER_T]
	}
}

unsafe impl<T> Native for Arc<T> {
	fn repr() -> AbiTypes {
		smallvec![POINTER_T]
	}
}

unsafe impl Native for RString {
	fn repr() -> AbiTypes {
		smallvec![POINTER_T]
	}
}
