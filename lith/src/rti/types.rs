//! Runtime type information.

use std::{marker::PhantomData, mem::ManuallyDrop};

use crate::rti;

pub struct Rtti {
	tag: RttiTag,
	data: RttiData,
}

impl Rtti {
	pub fn inner(&self) -> RttiRef {
		unsafe {
			match self.tag {
				RttiTag::Primitive => RttiRef::Primitive(&self.data.primitive),
				RttiTag::Struct => RttiRef::Struct(&self.data.structure),
			}
		}
	}
}

impl Clone for Rtti {
	fn clone(&self) -> Self {
		Self {
			tag: self.tag,
			data: unsafe {
				match self.tag {
					RttiTag::Primitive => RttiData {
						primitive: self.data.primitive,
					},
					RttiTag::Struct => RttiData {
						structure: self.data.structure.clone(),
					},
				}
			},
		}
	}
}

#[derive(Debug)]
pub enum RttiRef<'td> {
	Primitive(&'td PrimitiveType),
	Struct(&'td StructType),
}

// RttiData ////////////////////////////////////////////////////////////////////

/// Gets discriminated with [`RttiTag`].
union RttiData {
	structure: ManuallyDrop<StructType>,
	primitive: ManuallyDrop<PrimitiveType>,
}

/// Separated discriminant for [`RttiData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RttiTag {
	Primitive,
	Struct,
}

impl Drop for Rtti {
	fn drop(&mut self) {
		unsafe {
			match self.tag {
				RttiTag::Primitive => ManuallyDrop::drop(&mut self.data.primitive),
				RttiTag::Struct => ManuallyDrop::drop(&mut self.data.structure),
			}
		}
	}
}

impl std::fmt::Debug for Rtti {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unsafe {
			f.debug_struct("TypeDef")
				.field("tag", &self.tag)
				.field(
					"data",
					match &self.tag {
						RttiTag::Primitive => &self.data.primitive,
						RttiTag::Struct => &self.data.structure,
					},
				)
				.finish()
		}
	}
}

#[derive(Debug, Clone)]
pub struct FuncType {
	pub params: Vec<Parameter>,
	pub ret: rti::Handle<Rtti>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
	pub typeinfo: rti::Handle<Rtti>,
	pub optional: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveType {
	Bool,
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
	F32,
	F64,

	Void,
}

#[derive(Debug, Clone)]
pub struct StructType {}

// TypeHandle //////////////////////////////////////////////////////////////////

/// Specialization on [`crate::rti::Handle`].
#[derive(Debug, Clone)]
pub struct TypeHandle<T>(pub(crate) super::Handle<Rtti>, pub(crate) PhantomData<T>);

impl<T> PartialEq for TypeHandle<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<T> Eq for TypeHandle<T> {}

impl<T> TypeHandle<T> {
	#[must_use]
	pub fn upcast(self) -> rti::Handle<Rtti> {
		self.0
	}
}

// SAFETY: Whenever dereferencing `TypeHandle`, union accesses are guaranteed
// to be sound because a handle can not be created for the wrong type.

impl std::ops::Deref for TypeHandle<PrimitiveType> {
	type Target = PrimitiveType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.primitive }
	}
}

impl std::ops::Deref for TypeHandle<StructType> {
	type Target = StructType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.structure }
	}
}
