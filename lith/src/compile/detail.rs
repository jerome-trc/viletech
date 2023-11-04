use cranelift::prelude::AbiParam;

use crate::{
	front::sym::Symbol,
	types::{SymOPtr, SymPtr, TypeNPtr},
};

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

/// "Look-up table symbol". See [`crate::types::Scope`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LutSym {
	/// Exclusively for local variables, which don't need to be inserted
	/// into [`super::Compiler::symbols`].
	Owned {
		ptr: SymOPtr,
		imported: bool,
	},
	Unowned {
		ptr: SymPtr,
		imported: bool,
	},
}

impl LutSym {
	/// Separate from [`Self::non_owning_ptr`] to protect against accidentally
	/// misusing a `Self::Owned`.
	#[must_use]
	pub(crate) fn to_unowned(&self) -> Option<SymPtr> {
		match self {
			Self::Owned { ptr, .. } => None,
			Self::Unowned { ptr, .. } => Some(*ptr),
		}
	}

	#[must_use]
	pub(crate) fn non_owning_ptr(&self) -> SymPtr {
		match self {
			Self::Owned { ptr, .. } => SymPtr::from(ptr),
			Self::Unowned { ptr, .. } => *ptr,
		}
	}

	#[must_use]
	pub(crate) fn is_imported(&self) -> bool {
		match self {
			Self::Owned { imported, .. } => *imported,
			Self::Unowned { imported, .. } => *imported,
		}
	}
}

impl std::ops::Deref for LutSym {
	type Target = Symbol;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned { ptr, .. } => ptr,
			Self::Unowned { ptr, .. } => ptr,
		}
	}
}

impl Clone for LutSym {
	fn clone(&self) -> Self {
		match self {
			Self::Owned { ptr, imported } => Self::Unowned {
				ptr: SymPtr::from(ptr),
				imported: *imported,
			},
			Self::Unowned { ptr, imported } => Self::Unowned {
				ptr: *ptr,
				imported: *imported,
			},
		}
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
