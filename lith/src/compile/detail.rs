use cranelift::prelude::AbiParam;

use crate::types::{SymPtr, TypeNPtr};

use super::CEvalNative;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
	#[default]
	Registration,
	Declaration,
	Import,
	Sema,
	CodeGen,
}

/// "Look-up table symbol".
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LutSym {
	pub(crate) inner: SymPtr,
	pub(crate) imported: bool,
}

impl std::ops::Deref for LutSym {
	type Target = SymPtr;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

/// For use by [`crate::front::sema`].
#[derive(Debug)]
pub(crate) struct SymCache {
	pub(crate) void_t: TypeNPtr,
	pub(crate) bool_t: TypeNPtr,
	pub(crate) i8_t: TypeNPtr,
	pub(crate) u8_t: TypeNPtr,
	pub(crate) i16_t: TypeNPtr,
	pub(crate) u16_t: TypeNPtr,
	pub(crate) i32_t: TypeNPtr,
	pub(crate) u32_t: TypeNPtr,
	pub(crate) i64_t: TypeNPtr,
	pub(crate) u64_t: TypeNPtr,
	pub(crate) i128_t: TypeNPtr,
	pub(crate) u128_t: TypeNPtr,
	pub(crate) f32_t: TypeNPtr,
	pub(crate) f64_t: TypeNPtr,
	pub(crate) iname_t: TypeNPtr,
	pub(crate) never_t: TypeNPtr,
}

impl Default for SymCache {
	fn default() -> Self {
		Self {
			void_t: TypeNPtr::null(),
			bool_t: TypeNPtr::null(),
			i8_t: TypeNPtr::null(),
			u8_t: TypeNPtr::null(),
			i16_t: TypeNPtr::null(),
			u16_t: TypeNPtr::null(),
			i32_t: TypeNPtr::null(),
			u32_t: TypeNPtr::null(),
			i64_t: TypeNPtr::null(),
			u64_t: TypeNPtr::null(),
			i128_t: TypeNPtr::null(),
			u128_t: TypeNPtr::null(),
			f32_t: TypeNPtr::null(),
			f64_t: TypeNPtr::null(),
			iname_t: TypeNPtr::null(),
			never_t: TypeNPtr::null(),
		}
	}
}

#[derive(Debug)]
pub(crate) struct NativeFn {
	pub(crate) rt: Option<RuntimeNative>,
	pub(crate) ceval: Option<CEvalNative>,
}

#[derive(Debug)]
pub(crate) struct RuntimeNative {
	pub(crate) ptr: *const u8,
	pub(crate) params: &'static [AbiParam],
	pub(crate) returns: &'static [AbiParam],
}

unsafe impl Send for RuntimeNative {}
unsafe impl Sync for RuntimeNative {}
