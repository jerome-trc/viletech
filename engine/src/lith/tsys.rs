//! Symbols making up LithScript's type system.

use bitflags::bitflags;

use super::module::Handle;

/// Note that the type of a variable declared `let const x = 0` isn't separate
/// from the `i32` primitive. For qualified types such as that, see [`QualifiedType`].
#[derive(Debug)]
pub struct ScriptType {
	kind: TypeKind,
}

impl ScriptType {
	#[must_use]
	pub fn kind(&self) -> &TypeKind {
		&self.kind
	}
}

pub struct QualifiedType {
	inner: Handle<ScriptType>,
	quals: TypeQualifiers,
}

impl QualifiedType {
	#[must_use]
	pub fn inner(&self) -> &Handle<ScriptType> {
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
	I128,
	U128,
	Type,
	Array {
		value: Handle<ScriptType>,
		length: usize,
	},
	Class {
		ancestor: Option<Handle<ScriptType>>,
		structure: StructDesc,
		flags: ClassFlags,
	},
	Struct(StructDesc),
	Union {
		variants: Vec<StructDesc>,
	},
	Bitfield {
		/// Which integral type backs this bitfield?
		underlying: Handle<ScriptType>,
	},
	Pointer(Handle<ScriptType>),
	Reference(Handle<ScriptType>),
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
	stype: Handle<ScriptType>,
}

impl FieldDesc {
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn stype(&self) -> &Handle<ScriptType> {
		&self.stype
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
	bits: u128,
}

impl BitDesc {
	#[must_use]
	pub fn name(&self) -> &str {
		&self.name
	}

	#[must_use]
	pub fn affects_all(&self, bits: u128) -> bool {
		(self.bits & bits) == bits
	}
}
