//! The code that facilitates Rust/Lithica interoperability.

use std::hash::{Hash, Hasher};

use cranelift::prelude::types;

/// A pointer to a JIT function.
///
/// All implementors of this type are function pointers with only one return type,
/// since returning a stable-layout structure from a JIT function is always sound,
/// but passing an aggregate (struct, tuple, array) to one is never sound.
pub trait JitFn: 'static + Sized {
	fn sig_hash<H: Hasher>(state: &mut H);
}

impl<RET> JitFn for fn() -> RET
where
	RET: Native,
{
	fn sig_hash<H: Hasher>(state: &mut H) {
		RET::type_hash(state);
	}
}

macro_rules! impl_jitfn {
	($($($gen:ident),+);+) => {
		$(
			impl<RET, $($gen),+> JitFn for fn($($gen),+) -> RET
			where
				RET: Native,
				$($gen: Native),+
			{
				fn sig_hash<H: Hasher>(state: &mut H) {
					RET::type_hash(state);

					$(
						$gen::type_hash(state);
					)+
				}
			}
		)+
	};
}

impl_jitfn! {
	A;
	A, B;
	A, B, C;
	A, B, C, D;
	A, B, C, D, E;
	A, B, C, D, E, F
}

// (RAT) Why does Rust not have variadic generics again?

/// # Safety
///
/// This type is not meant for implementation by user code, since only primitive
/// types can be soundly passed between the Rust/Lithica ABI boundary, and any
/// implementations you should need are already provided.
pub unsafe trait Native: 'static + Sized {
	fn type_hash<H: Hasher>(state: &mut H);
}

unsafe impl Native for () {
	fn type_hash<H: Hasher>(_: &mut H) {}
}

unsafe impl Native for bool {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I8.hash(state);
	}
}

unsafe impl Native for char {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I32.hash(state);
	}
}

unsafe impl Native for i8 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I8.hash(state);
	}
}

unsafe impl Native for i16 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I16.hash(state);
	}
}

unsafe impl Native for i32 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I32.hash(state);
	}
}

unsafe impl Native for i64 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I64.hash(state);
	}
}

unsafe impl Native for i128 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I128.hash(state);
	}
}

unsafe impl Native for u8 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I8.hash(state);
	}
}

unsafe impl Native for u16 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I16.hash(state);
	}
}

unsafe impl Native for u32 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I32.hash(state);
	}
}

unsafe impl Native for u64 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I64.hash(state);
	}
}

unsafe impl Native for u128 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::I128.hash(state);
	}
}

unsafe impl Native for isize {
	fn type_hash<H: Hasher>(state: &mut H) {
		#[cfg(target_pointer_width = "64")]
		types::I64.hash(state);
		#[cfg(target_pointer_width = "32")]
		types::I32.hash(state);
	}
}

unsafe impl Native for usize {
	fn type_hash<H: Hasher>(state: &mut H) {
		isize::type_hash(state);
	}
}

unsafe impl Native for f32 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::F32.hash(state);
	}
}

unsafe impl Native for f64 {
	fn type_hash<H: Hasher>(state: &mut H) {
		types::F64.hash(state);
	}
}

unsafe impl<T: 'static + Sized> Native for *mut T {
	fn type_hash<H: Hasher>(state: &mut H) {
		isize::type_hash(state);
	}
}

unsafe impl<T: 'static + Sized> Native for *const T {
	fn type_hash<H: Hasher>(state: &mut H) {
		isize::type_hash(state);
	}
}
