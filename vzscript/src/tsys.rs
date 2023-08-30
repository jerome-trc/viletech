//! Type information, used for compilation as well as RTTI.

use std::{alloc::Layout, marker::PhantomData, mem::ManuallyDrop};

use crate::rti;

/// No VZScript type is allowed to exceed this size in bytes.
pub const MAX_SIZE: usize = 1024 * 2;

pub struct TypeDef {
	tag: TypeTag,
	data: TypeData,
	layout: Layout,
}

impl rti::RtInfo for TypeDef {}

impl TypeDef {
	#[must_use]
	pub fn layout(&self) -> Layout {
		self.layout
	}

	pub fn inner(&self) -> TypeRef {
		unsafe {
			match self.tag {
				TypeTag::Array => TypeRef::Array(&self.data.array),
				TypeTag::Class => TypeRef::Class(&self.data.class),
				TypeTag::Function => TypeRef::Function(&self.data.func),
				TypeTag::Numeric => TypeRef::Num(&self.data.numeric),
				TypeTag::Struct => TypeRef::Struct(&self.data.structure),
				TypeTag::TypeDef => TypeRef::TypeDef(&self.data.typedef),
				TypeTag::Union => TypeRef::Union(&self.data.union_),
				TypeTag::Void => TypeRef::Void(&self.data.void),
			}
		}
	}

	#[must_use]
	pub(crate) fn new_array(array_t: ArrayType) -> Self {
		Self {
			layout: {
				let e_layout = array_t.elem.upgrade().layout();
				Layout::from_size_align(e_layout.size() * array_t.len, 16).unwrap()
			},
			tag: TypeTag::Array,
			data: TypeData {
				array: ManuallyDrop::new(array_t),
			},
		}
	}

	#[must_use]
	pub(crate) fn new_class(class_t: StructType) -> Self {
		Self {
			tag: TypeTag::Struct,
			data: TypeData {
				structure: ManuallyDrop::new(class_t),
			},
			layout: unimplemented!(),
		}
	}

	pub(crate) const BUILTINS: &[Self] = &[
		Self {
			tag: TypeTag::TypeDef,
			data: TypeData {
				typedef: ManuallyDrop::new(TypeDefType),
			},
			layout: Layout::new::<()>(),
		},
		Self {
			tag: TypeTag::Void,
			data: TypeData {
				void: ManuallyDrop::new(VoidType),
			},
			layout: Layout::new::<()>(),
		},
		// Numeric /////////////////////////////////////////////////////////////
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::I8),
			},
			layout: Layout::new::<i8>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::U8),
			},
			layout: Layout::new::<u8>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::I16),
			},
			layout: Layout::new::<i16>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::U16),
			},
			layout: Layout::new::<u16>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::I32),
			},
			layout: Layout::new::<i32>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::U32),
			},
			layout: Layout::new::<u32>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::I64),
			},
			layout: Layout::new::<i64>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::U64),
			},
			layout: Layout::new::<u64>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::F32),
			},
			layout: Layout::new::<f32>(),
		},
		Self {
			tag: TypeTag::Numeric,
			data: TypeData {
				numeric: ManuallyDrop::new(NumType::F64),
			},
			layout: Layout::new::<f64>(),
		},
	];
}

#[derive(Debug)]
pub enum TypeRef<'td> {
	Array(&'td ArrayType),
	Class(&'td ClassType),
	Function(&'td FuncType),
	Num(&'td NumType),
	Struct(&'td StructType),
	TypeDef(&'td TypeDefType),
	Union(&'td UnionType),
	Void(&'td VoidType),
}

// TypeData ////////////////////////////////////////////////////////////////////

/// Gets discriminated with [`TypeTag`].
union TypeData {
	array: ManuallyDrop<ArrayType>,
	class: ManuallyDrop<ClassType>,
	func: ManuallyDrop<FuncType>,
	numeric: ManuallyDrop<NumType>,
	structure: ManuallyDrop<StructType>,
	typedef: ManuallyDrop<TypeDefType>,
	union_: ManuallyDrop<UnionType>,
	void: ManuallyDrop<VoidType>,
}

/// Separated discriminant for [`TypeData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TypeTag {
	Array,
	Class,
	Function,
	Numeric,
	Struct,
	TypeDef,
	Union,
	Void,
}

impl Drop for TypeDef {
	fn drop(&mut self) {
		unsafe {
			match self.tag {
				TypeTag::Array => ManuallyDrop::drop(&mut self.data.array),
				TypeTag::Class => ManuallyDrop::drop(&mut self.data.class),
				TypeTag::Function => ManuallyDrop::drop(&mut self.data.func),
				TypeTag::Numeric => ManuallyDrop::drop(&mut self.data.numeric),
				TypeTag::Struct => ManuallyDrop::drop(&mut self.data.structure),
				TypeTag::TypeDef => ManuallyDrop::drop(&mut self.data.typedef),
				TypeTag::Union => ManuallyDrop::drop(&mut self.data.union_),
				TypeTag::Void => ManuallyDrop::drop(&mut self.data.void),
			}
		}
	}
}

impl std::fmt::Debug for TypeDef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		unsafe {
			f.debug_struct("TypeDef")
				.field("tag", &self.tag)
				.field(
					"data",
					match &self.tag {
						TypeTag::Array => &self.data.array,
						TypeTag::Class => &self.data.class,
						TypeTag::Function => &self.data.func,
						TypeTag::Numeric => &self.data.numeric,
						TypeTag::Struct => &self.data.structure,
						TypeTag::TypeDef => &self.data.typedef,
						TypeTag::Union => &self.data.union_,
						TypeTag::Void => &self.data.void,
					},
				)
				.finish()
		}
	}
}

// TypeData's contents /////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ArrayType {
	pub len: usize,
	pub elem: rti::InHandle<TypeDef>,
}

#[derive(Debug)]
pub struct ClassType;

#[derive(Debug)]
pub struct EnumType;

#[derive(Debug)]
pub struct FuncType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumType {
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

#[derive(Debug)]
pub struct StructType;

#[derive(Debug)]
pub struct TypeDefType;

#[derive(Debug)]
pub struct UnionType;

#[derive(Debug)]
pub struct VoidType;

// TypeHandle //////////////////////////////////////////////////////////////////

/// Specialization on [`crate::rti::Handle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeHandle<T>(rti::Handle<TypeDef>, PhantomData<T>);

// SAFETY: Whenever dereferencing `TypeHandle`, union accesses are guaranteed
// to be sound because a handle can not be created for the wrong type.

impl std::ops::Deref for TypeHandle<ArrayType> {
	type Target = ArrayType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.array }
	}
}

impl std::ops::Deref for TypeHandle<ClassType> {
	type Target = ClassType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.class }
	}
}

impl std::ops::Deref for TypeHandle<FuncType> {
	type Target = FuncType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.func }
	}
}

impl std::ops::Deref for TypeHandle<NumType> {
	type Target = NumType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.numeric }
	}
}

impl std::ops::Deref for TypeHandle<StructType> {
	type Target = StructType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.structure }
	}
}

impl std::ops::Deref for TypeHandle<TypeDefType> {
	type Target = TypeDefType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.typedef }
	}
}

impl std::ops::Deref for TypeHandle<UnionType> {
	type Target = UnionType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.union_ }
	}
}

impl std::ops::Deref for TypeHandle<VoidType> {
	type Target = VoidType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.void }
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInHandle<T>(rti::InHandle<TypeDef>, PhantomData<T>);
