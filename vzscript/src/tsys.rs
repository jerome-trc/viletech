//! Type information, used for compilation as well as RTTI.

use std::{alloc::Layout, marker::PhantomData, mem::ManuallyDrop};

use util::rstring::RString;

use crate::{rti, zname::ZName};

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
				TypeTag::IName => TypeRef::IName(&self.data.iname),
				TypeTag::Numeric => TypeRef::Num(&self.data.numeric),
				TypeTag::String => TypeRef::String(&self.data.string),
				TypeTag::Struct => TypeRef::Struct(&self.data.structure),
				TypeTag::TypeDef => TypeRef::TypeDef(&self.data.typedef),
				TypeTag::Union => TypeRef::Union(&self.data.r#union),
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

	pub(crate) const BUILTINS: &[(&'static str, Self)] = &[
		(
			"vzscript::typedef",
			Self {
				tag: TypeTag::TypeDef,
				data: TypeData {
					typedef: ManuallyDrop::new(TypeDefType),
				},
				layout: Layout::new::<rti::Handle<TypeDef>>(),
			},
		),
		(
			"vzscript::void",
			Self {
				tag: TypeTag::Void,
				data: TypeData {
					void: ManuallyDrop::new(VoidType),
				},
				layout: Layout::new::<()>(),
			},
		),
		// Numeric /////////////////////////////////////////////////////////////
		(
			"vzscript::int8",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Int8),
				},
				layout: Layout::new::<i8>(),
			},
		),
		(
			"vzscript::uint8",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Uint8),
				},
				layout: Layout::new::<u8>(),
			},
		),
		(
			"vzscript::int16",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Int16),
				},
				layout: Layout::new::<i16>(),
			},
		),
		(
			"vzscript::uint16",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Uint16),
				},
				layout: Layout::new::<u16>(),
			},
		),
		(
			"vzscript::int32",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Int32),
				},
				layout: Layout::new::<i32>(),
			},
		),
		(
			"vzscript::uint32",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Uint32),
				},
				layout: Layout::new::<u32>(),
			},
		),
		(
			"vzscript::int64",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Iint64),
				},
				layout: Layout::new::<i64>(),
			},
		),
		(
			"vzscript::uint64",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Uint64),
				},
				layout: Layout::new::<u64>(),
			},
		),
		(
			"vzscript::int128",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Int128),
				},
				layout: Layout::new::<i128>(),
			},
		),
		(
			"vzscript::uint128",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Uint128),
				},
				layout: Layout::new::<u128>(),
			},
		),
		(
			"vzscript::float",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Float32),
				},
				layout: Layout::new::<f32>(),
			},
		),
		(
			"vzscript::float64",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					numeric: ManuallyDrop::new(NumType::Float64),
				},
				layout: Layout::new::<f64>(),
			},
		),
		(
			"vzscript::string",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					string: ManuallyDrop::new(StringType),
				},
				layout: Layout::new::<RString>(),
			},
		),
		(
			"vzscript::iname",
			Self {
				tag: TypeTag::Numeric,
				data: TypeData {
					iname: ManuallyDrop::new(INameType),
				},
				layout: Layout::new::<ZName>(),
			},
		),
	];
}

impl Clone for TypeDef {
	fn clone(&self) -> Self {
		Self {
			tag: self.tag,
			data: unsafe {
				match self.tag {
					TypeTag::Array => TypeData {
						array: self.data.array.clone(),
					},
					TypeTag::Class => TypeData {
						class: self.data.class.clone(),
					},
					TypeTag::Function => TypeData {
						func: self.data.func.clone(),
					},
					TypeTag::IName => TypeData {
						iname: self.data.iname.clone(),
					},
					TypeTag::Numeric => TypeData {
						numeric: self.data.numeric,
					},
					TypeTag::String => TypeData {
						string: self.data.string.clone(),
					},
					TypeTag::Struct => TypeData {
						structure: self.data.structure.clone(),
					},
					TypeTag::TypeDef => TypeData {
						typedef: self.data.typedef.clone(),
					},
					TypeTag::Union => TypeData {
						r#union: self.data.union.clone(),
					},
					TypeTag::Void => TypeData {
						void: self.data.void.clone(),
					},
				}
			},
			layout: self.layout,
		}
	}
}

#[derive(Debug)]
pub enum TypeRef<'td> {
	Array(&'td ArrayType),
	Class(&'td ClassType),
	Function(&'td FuncType),
	IName(&'td INameType),
	Num(&'td NumType),
	String(&'td StringType),
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
	iname: ManuallyDrop<INameType>,
	numeric: ManuallyDrop<NumType>,
	string: ManuallyDrop<StringType>,
	structure: ManuallyDrop<StructType>,
	typedef: ManuallyDrop<TypeDefType>,
	r#union: ManuallyDrop<UnionType>,
	void: ManuallyDrop<VoidType>,
}

/// Separated discriminant for [`TypeData`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeTag {
	Array,
	Class,
	Function,
	IName,
	Numeric,
	String,
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
				TypeTag::IName => ManuallyDrop::drop(&mut self.data.iname),
				TypeTag::Numeric => ManuallyDrop::drop(&mut self.data.numeric),
				TypeTag::Struct => ManuallyDrop::drop(&mut self.data.structure),
				TypeTag::String => ManuallyDrop::drop(&mut self.data.string),
				TypeTag::TypeDef => ManuallyDrop::drop(&mut self.data.typedef),
				TypeTag::Union => ManuallyDrop::drop(&mut self.data.r#union),
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
						TypeTag::IName => &self.data.iname,
						TypeTag::Numeric => &self.data.numeric,
						TypeTag::String => &self.data.string,
						TypeTag::Struct => &self.data.structure,
						TypeTag::TypeDef => &self.data.typedef,
						TypeTag::Union => &self.data.r#union,
						TypeTag::Void => &self.data.void,
					},
				)
				.finish()
		}
	}
}

// TypeData's contents /////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ArrayType {
	pub len: usize,
	pub elem: rti::InHandle<TypeDef>,
}

#[derive(Debug, Clone)]
pub struct ClassType {
	pub restrict: rti::Restriction,
}

#[derive(Debug, Clone)]
pub struct EnumType;

#[derive(Debug, Clone)]
pub struct FuncType;

#[derive(Debug, Clone)]
pub struct INameType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumType {
	Int8,
	Uint8,
	Int16,
	Uint16,
	Int32,
	Uint32,
	Iint64,
	Uint64,
	Int128,
	Uint128,
	Float32,
	Float64,
}

#[derive(Debug, Clone)]
pub struct StringType;

#[derive(Debug, Clone)]
pub struct StructType;

#[derive(Debug, Clone)]
pub struct TypeDefType;

#[derive(Debug, Clone)]
pub struct UnionType;

#[derive(Debug, Clone)]
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

impl std::ops::Deref for TypeHandle<INameType> {
	type Target = INameType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.iname }
	}
}

impl std::ops::Deref for TypeHandle<NumType> {
	type Target = NumType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.numeric }
	}
}

impl std::ops::Deref for TypeHandle<StringType> {
	type Target = StringType;

	fn deref(&self) -> &Self::Target {
		unsafe { &self.0.data.string }
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
		unsafe { &self.0.data.r#union }
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
