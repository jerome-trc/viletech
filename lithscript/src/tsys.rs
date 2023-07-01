//! Type information, used for compilation as well as RTTI.

use std::{alloc::Layout, marker::PhantomData, mem::ManuallyDrop};

use crate::rti;

/// No LithScript type is allowed to exceed this size in bytes.
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

	pub fn inner(&self) -> TypeInfo {
		unsafe {
			match self.tag {
				TypeTag::Array => TypeInfo::Array(&self.data.array),
				TypeTag::Class => TypeInfo::Class(&self.data.class),
				TypeTag::Function => TypeInfo::Function(&self.data.func),
				TypeTag::Numeric => TypeInfo::Num(&self.data.numeric),
				TypeTag::TypeDef => TypeInfo::TypeDef(&self.data.typedef),
				TypeTag::Void => TypeInfo::Void(&self.data.void),
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
	pub(crate) fn new_class(class_t: ClassType) -> Self {
		Self {
			tag: TypeTag::Class,
			data: TypeData {
				class: ManuallyDrop::new(class_t),
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
pub enum TypeInfo<'td> {
	Array(&'td ArrayType),
	Class(&'td ClassType),
	Function(&'td FuncType),
	Num(&'td NumType),
	TypeDef(&'td TypeDefType),
	Void(&'td VoidType),
}

#[derive(Debug)]
pub struct VoidType;
#[derive(Debug)]
pub struct TypeDefType;

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
pub struct ArrayType {
	pub len: usize,
	pub elem: rti::InHandle<TypeDef>,
}

#[derive(Debug)]
pub struct ClassType {
	pub parent: Option<TypeInHandle<ClassType>>,
}

#[derive(Debug)]
pub struct FuncType;

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

impl std::ops::Deref for TypeHandle<TypeDefType> {
	type Target = TypeDefType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.typedef }
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

/// Gets discriminated with [`TypeTag`].
union TypeData {
	array: ManuallyDrop<ArrayType>,
	class: ManuallyDrop<ClassType>,
	func: ManuallyDrop<FuncType>,
	numeric: ManuallyDrop<NumType>,
	typedef: ManuallyDrop<TypeDefType>,
	void: ManuallyDrop<VoidType>,
}

/// Separated discriminant for [`TypeData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TypeTag {
	Array,
	Class,
	Function,
	Numeric,
	TypeDef,
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
				TypeTag::TypeDef => ManuallyDrop::drop(&mut self.data.typedef),
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
						TypeTag::TypeDef => &self.data.typedef,
						TypeTag::Void => &self.data.void,
					},
				)
				.finish()
		}
	}
}
