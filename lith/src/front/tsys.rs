//! Data structures for representing Lithica types in the frontend.

use std::sync::atomic::{self, AtomicUsize};

use cranelift::codegen::ir::types as abi_t;
use smallvec::{smallvec, SmallVec};

use crate::{
	data::Visibility,
	types::{AbiTypes, TypeNPtr},
};

#[derive(Debug)]
pub(crate) enum TypeDef {
	Primitive(Primitive),
	Structure(Structure),
}

/// Short for "frontend type".
/// Used to represent type specifications prior to any monomorphization.
#[derive(Debug)]
pub(crate) enum FrontType {
	Any {
		optional: bool,
	},
	Type {
		array_dims: SmallVec<[ArrayLength; 1]>,
		optional: bool,
	},
	Normal(SemaType),
}

#[derive(Debug)]
pub(crate) struct SemaType {
	pub(crate) inner: TypeNPtr,
	pub(crate) array_dims: SmallVec<[ArrayLength; 1]>,
	pub(crate) optional: bool,
	pub(crate) reference: bool,
}

#[derive(Debug)]
pub(crate) struct ArrayLength(AtomicUsize);

impl ArrayLength {
	#[must_use]
	pub(crate) fn get(&self) -> usize {
		let ret = self.0.load(atomic::Ordering::Acquire);
		debug_assert_ne!(ret, 0);
		ret
	}

	pub(crate) fn set(&self, len: usize) {
		debug_assert_eq!(self.0.load(atomic::Ordering::Acquire), 0);
		debug_assert_ne!(len, 0);
		self.0.store(len, atomic::Ordering::Release);
	}
}

impl Default for ArrayLength {
	fn default() -> Self {
		Self(AtomicUsize::new(0))
	}
}

impl PartialEq for ArrayLength {
	fn eq(&self, other: &Self) -> bool {
		self.get() == other.get()
	}
}

impl Eq for ArrayLength {}

// Primitive ///////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Primitive {
	Never,
	Void,
	Bool,
	I8,
	I16,
	I32,
	I64,
	I128,
	U8,
	U16,
	U32,
	U64,
	U128,
	F32,
	F64,
}

impl Primitive {
	#[must_use]
	pub(crate) fn abi(self) -> AbiTypes {
		match self {
			Self::Never | Self::Void => smallvec![],
			Self::Bool | Self::I8 | Self::U8 => smallvec![abi_t::I8],
			Self::I16 | Self::U16 => smallvec![abi_t::I16],
			Self::I32 | Self::U32 => smallvec![abi_t::I32],
			Self::I64 | Self::U64 => smallvec![abi_t::I64],
			Self::I128 | Self::U128 => smallvec![abi_t::I128],
			Self::F32 => smallvec![abi_t::F32],
			Self::F64 => smallvec![abi_t::F64],
		}
	}
}

// Structure ///////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct Structure {
	pub(crate) fields: Vec<Field>,
}

#[derive(Debug)]
pub(crate) struct Field {
	pub(crate) vis: Visibility,
}
