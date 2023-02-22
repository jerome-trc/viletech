//! Symbols making up LithScript's type system.

use std::any::TypeId;

use bitflags::bitflags;

use crate::lith::{abi::QWord, heap};

use super::{
	module::{Handle, InHandle},
	Symbol,
};

/// No LithScript type is allowed to exceed this size in bytes.
pub const MAX_SIZE: usize = 1024 * 2;

/// Note that the type of a variable declared `let const x = 0` isn't separate
/// from the `i32` primitive. For qualified types such as that, see [`QualifiedType`].
#[derive(Debug)]
pub struct TypeInfo {
	kind: TypeKind,
	native: Option<NativeInfo>,
	/// See the documentation for the method of the same name.
	layout: std::alloc::Layout,
	/// See the documentation for the method of the same name.
	heap_layout: std::alloc::Layout,
}

impl TypeInfo {
	pub fn native(&self) -> &Option<NativeInfo> {
		&self.native
	}

	#[must_use]
	pub fn kind(&self) -> &TypeKind {
		&self.kind
	}

	#[must_use]
	pub fn layout(&self) -> std::alloc::Layout {
		self.layout
	}

	/// This includes the allocation's header; it is always 16-byte aligned and
	/// always at least 16 bytes large, except for zero-size types.
	#[must_use]
	pub fn heap_layout(&self) -> std::alloc::Layout {
		self.heap_layout
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NativeInfo {
	type_id: TypeId,
	size: usize,
	align: usize,
}

impl NativeInfo {
	#[must_use]
	fn new<T: Sized + 'static>() -> Self {
		Self {
			type_id: TypeId::of::<T>(),
			size: std::mem::size_of::<T>(),
			align: std::mem::align_of::<T>(),
		}
	}
}

impl Symbol for TypeInfo {}

#[derive(Debug)]
pub struct QualifiedType {
	inner: Handle<TypeInfo>,
	quals: TypeQualifiers,
}

impl QualifiedType {
	#[must_use]
	pub fn inner(&self) -> &Handle<TypeInfo> {
		&self.inner
	}

	#[must_use]
	pub fn qualifiers(&self) -> TypeQualifiers {
		self.quals
	}
}

bitflags! {
	pub struct TypeQualifiers: u8 {
		const CONST = 1 << 0;
	}
}

#[derive(Debug)]
pub enum TypeKind {
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	TypeInfo,
	Array {
		value: InHandle<TypeInfo>,
		length: usize,
	},
	Class {
		ancestor: Option<InHandle<TypeInfo>>,
		structure: StructDesc,
		flags: ClassFlags,
	},
	Struct(StructDesc),
	Union {
		variants: Vec<StructDesc>,
	},
	Bitfield {
		/// Which integral type backs this bitfield?
		underlying: InHandle<TypeInfo>,
	},
	Pointer(InHandle<TypeInfo>),
	Reference(InHandle<TypeInfo>),
}

#[derive(Debug)]
pub struct StructDesc {
	fields: Vec<FieldDesc>,
}

impl StructDesc {
	#[must_use]
	pub fn fields(&self) -> &[FieldDesc] {
		&self.fields
	}
}

bitflags! {
	pub struct ClassFlags: u8 {
		/// This class can't be inherited from. Mutually exclusive with `ABSTRACT`.
		const FINAL = 1 << 0;
		/// This class can't be instiantiated; it's only a base for inheritance.
		/// Mutually exclusive with `FINAL`.
		const ABSTRACT = 1 << 1;
	}
}

#[derive(Debug)]
pub struct FieldDesc {
	/// Human-readable.
	name: String,
	tinfo: InHandle<TypeInfo>,
	/// See the documentation for the method of the same name.
	offset: usize,
}

impl FieldDesc {
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn tinfo(&self) -> &InHandle<TypeInfo> {
		&self.tinfo
	}

	/// In bytes, from the end of the allocation header.
	#[must_use]
	#[allow(unused)]
	pub(super) fn offset(&self) -> usize {
		self.offset
	}
}

#[derive(Debug)]
pub struct VariantDesc {
	name: String,
	structure: StructDesc,
}

impl VariantDesc {
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn structure(&self) -> &StructDesc {
		&self.structure
	}
}

/// One subfield in a bitfield.
#[derive(Debug)]
pub struct BitDesc {
	/// Human-readable.
	name: String,
	/// Operations on this subfield alter all of the bits set on this integer.
	bits: u64,
}

impl BitDesc {
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn affects_all(&self, bits: u64) -> bool {
		(self.bits & bits) == bits
	}
}

/// For use when constructing the `lith` module.
#[must_use]
pub(super) fn builtins() -> Vec<(String, TypeInfo)> {
	use std::alloc::Layout;

	let qword_layout = Layout::new::<QWord>();
	let qword_heap_layout = heap::layout_for(qword_layout);

	vec![
		(
			"i32".to_string(),
			TypeInfo {
				kind: TypeKind::I32,
				native: Some(NativeInfo::new::<i32>()),
				layout: qword_layout,
				heap_layout: qword_heap_layout,
			},
		),
		// TODO: ...everything else.
	]
}
