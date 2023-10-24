//! Data structures for representing Lithica types in the frontend.

use cranelift::codegen::ir::types as abi_t;

use crate::{
	intern::NameIx,
	types::{AbiType, TypeNPtr, TypePtr},
};

use super::sym::Visibility;

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum TypeDatum {
	Array { inner: TypePtr, len: usize },
	Primitive(Primitive),
	Structure(Structure),
}

// Primitive ///////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
	IName,
}

impl Primitive {
	#[must_use]
	pub(crate) fn abi(self) -> Option<AbiType> {
		match self {
			Self::Never | Self::Void => None,
			Self::Bool | Self::I8 | Self::U8 => Some(abi_t::I8),
			Self::I16 | Self::U16 => Some(abi_t::I16),
			Self::I32 | Self::U32 | Self::IName => Some(abi_t::I32),
			Self::I64 | Self::U64 => Some(abi_t::I64),
			Self::I128 | Self::U128 => Some(abi_t::I128),
			Self::F32 => Some(abi_t::F32),
			Self::F64 => Some(abi_t::F64),
		}
	}
}

// Structure ///////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Structure {
	pub(crate) fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Field {
	pub(crate) name: NameIx,
	pub(crate) tspec: TypeNPtr,
	pub(crate) _visibility: Visibility,
}
