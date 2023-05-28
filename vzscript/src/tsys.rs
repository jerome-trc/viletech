//! Infrastructure supporting VZScript's type system.

use std::{alloc::Layout, sync::Arc};

use crate::sym::{self, Symbol, SymbolKey, SymbolStore, TypeInHandle};

/// No VZScript type is allowed to exceed this size in bytes.
pub const MAX_SIZE: usize = 1024 * 2;

/// An implementation detail that also provides a constraint on [`TypeInfo`]'s
/// generic parameter.
pub trait TypeData: Send + Sync {
	const GROUP: TypeGroup;
}

/// An implementation detail that also provides a constraint on [`TypeInfo`]'s
/// generic parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeGroup {
	Void,
	Numeric,
	Array,
}

#[derive(Debug)]
pub struct TypeInfo<T: TypeData> {
	/// See the documentation for the method of the same name.
	layout: Layout,
	inner: T,
}

impl<T: TypeData> TypeInfo<T> {
	/// The layout of the type itself. Not necessarily 16-byte aligned.
	#[must_use]
	pub fn layout(&self) -> Layout {
		self.layout
	}
}

impl<T: TypeData> std::ops::Deref for TypeInfo<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl TypeData for () {
	const GROUP: TypeGroup = TypeGroup::Void;
}

impl Symbol for TypeInfo<()> {
	type HashInput<'i> = ();

	fn key<'j>(&self) -> SymbolKey {
		SymbolKey::new::<Self>(())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Numeric {
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	F32,
	F64,
}

impl Numeric {
	#[must_use]
	pub const fn name(&self) -> &'static str {
		match self {
			Self::I8 => "int8",
			Self::U8 => "uint8",
			Self::I16 => "int16",
			Self::U16 => "uint16",
			Self::I32 => "int32",
			Self::U32 => "uint32",
			Self::I64 => "int64",
			Self::U64 => "uint64",
			Self::F32 => "float32",
			Self::F64 => "float64",
		}
	}
}

impl TypeData for Numeric {
	const GROUP: TypeGroup = TypeGroup::Numeric;
}

impl Symbol for TypeInfo<Numeric> {
	type HashInput<'i> = &'i str;

	fn key<'j>(&self) -> SymbolKey {
		SymbolKey::new::<Self>(self.inner.name())
	}
}

#[derive(Debug)]
pub struct Array {
	elem_type: TypeInHandle,
	len: usize,
}

impl Array {
	#[must_use]
	pub fn elem_type(&self) -> &TypeInHandle {
		&self.elem_type
	}

	#[must_use]
	pub fn length(&self) -> usize {
		self.len
	}
}

impl TypeData for Array {
	const GROUP: TypeGroup = TypeGroup::Array;
}

impl Symbol for TypeInfo<Array> {
	type HashInput<'i> = (usize, &'i str);

	fn key<'j>(&self) -> SymbolKey {
		let thandle = self.inner.elem_type.upgrade();
		let input = (self.length(), thandle.name());
		SymbolKey::new::<Self>(input)
	}
}

/// All the types that make up the corelib
/// but which are not directly declared in script files.
#[must_use]
pub(super) fn _builtins() -> Vec<Arc<dyn SymbolStore>> {
	let qword_layout = Layout::new::<i64>();

	vec![
		Arc::new(sym::Store::new(
			Numeric::I32.name().to_string(),
			TypeInfo {
				layout: qword_layout,
				inner: Numeric::I32,
			},
		)), // TODO: The rest of them.
	]
}
