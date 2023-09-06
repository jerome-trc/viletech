//! Information used by the semantic mid-section but not the backend.

use std::any::Any;

use arc_swap::ArcSwap;
use doomfront::rowan::{GreenNode, TextRange};

use crate::{
	rti,
	tsys::{ClassType, EnumType, FuncType, StructType, TypeDef, TypeHandle, UnionType},
	vir,
};

use super::{
	intern::{PathIx, SymbolIx},
	Scope,
};

#[derive(Debug)]
pub(crate) struct Symbol {
	/// `None` for primitives and namespaces.
	pub(crate) location: Option<Location>,
	/// `None` for primitives and namespaces.
	pub(crate) source: Option<GreenNode>,
	pub(crate) def: Definition,
	/// Determines whether name lookups need to happen through all namespaces
	/// (to imitate the reference implementation's global namespace) or just
	/// the library's own.
	pub(crate) zscript: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Location {
	pub(crate) file: PathIx,
	pub(crate) span: TextRange,
}

#[derive(Debug)]
pub(crate) enum Definition {
	Class(Box<ClassDef>),
	Enum(Box<EnumDef>),
	FlagDef,
	Function(Box<FunctionDef>),
	Mixin,
	Primitive(Box<Primitive>),
	Property,
	/// Primarily for VZS' type aliasing feature, but also for transpilation.
	Rename {
		aliased: SymbolIx,
	},
	Struct(Box<StructDef>),
	Union(Box<UnionDef>),
	Value(Box<ValueDef>),
	None {
		kind: Undefined,
		extra: Box<dyn 'static + Any + Send + Sync>,
	},
	Pending,
	Error,
}

#[derive(Debug)]
pub(crate) struct ClassDef {
	pub(crate) tdef: TypeHandle<ClassType>,
	pub(crate) scope: Scope,
}

#[derive(Debug)]
pub(crate) struct EnumDef {
	pub(crate) tdef: TypeHandle<EnumType>,
	pub(crate) scope: Scope,
}

#[derive(Debug)]
pub(crate) struct FunctionDef {
	pub(crate) tdef: TypeHandle<FuncType>,
	pub(crate) code: vir::Block,
}

#[derive(Debug)]
pub(crate) struct Primitive {
	pub(crate) tdef: rti::Handle<TypeDef>,
	pub(crate) scope: Scope,
}

#[derive(Debug)]
pub(crate) struct StructDef {
	pub(crate) typedef: TypeHandle<StructType>,
	pub(crate) scope: Scope,
}

#[derive(Debug)]
pub(crate) struct UnionDef {
	pub(crate) typedef: TypeHandle<UnionType>,
	pub(crate) scope: Scope,
}

#[derive(Debug)]
pub(crate) struct ValueDef {
	pub(crate) typedef: rti::Handle<TypeDef>,
	pub(crate) init: vir::Block,
}

#[derive(Debug)]
pub(crate) enum FunctionCode {
	Native(Box<str>),
	Ir(vir::Block),
}

impl Symbol {
	#[must_use]
	pub(crate) fn scope(&self) -> Option<&Scope> {
		match &self.def {
			Definition::Class(def) => Some(&def.scope),
			Definition::Enum(def) => Some(&def.scope),
			Definition::Primitive(def) => Some(&def.scope),
			Definition::Struct(def) => Some(&def.scope),
			Definition::Union(def) => Some(&def.scope),
			Definition::Pending
			| Definition::Error
			| Definition::FlagDef
			| Definition::Function { .. }
			| Definition::Mixin { .. }
			| Definition::None { .. }
			| Definition::Property
			| Definition::Rename { .. }
			| Definition::Value { .. } => None,
		}
	}

	#[must_use]
	pub(crate) fn is_undefined(&self) -> bool {
		matches!(&self.def, Definition::None { .. })
	}

	#[must_use]
	pub(crate) fn definition_pending(&self) -> bool {
		matches!(&self.def, Definition::Pending)
	}

	#[must_use]
	pub(crate) fn is_defined(&self) -> bool {
		!(self.is_undefined() || self.definition_pending())
	}
}

impl Definition {
	#[must_use]
	pub(crate) fn user_facing_name(&self) -> &'static str {
		match self {
			Self::Class { .. } => "class",
			Self::Enum { .. } => "enum",
			Self::FlagDef => "flagdef",
			Self::Function { .. } => "function",
			Self::Mixin { .. } => "mixin",
			Self::Primitive { .. } => "primitive",
			Self::Property => "property",
			Self::Struct { .. } => "struct",
			Self::Rename { .. } => "type alias",
			Self::Union { .. } => "union",
			Self::Value { .. } => "value",
			Self::None { kind, .. } => match kind {
				Undefined::Class => "class",
				Undefined::Enum => "enum",
				Undefined::FlagDef => "flagdef",
				Undefined::Function => "function",
				Undefined::Property => "property",
				Undefined::Struct => "struct",
				Undefined::Union => "union",
				Undefined::Value => "value",
			},
			Self::Error | Self::Pending => unreachable!(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Undefined {
	Class,
	Enum,
	FlagDef,
	Function,
	Property,
	Struct,
	Union,
	Value,
}

pub(crate) type SymbolPtr = ArcSwap<Symbol>;

impl From<Symbol> for SymbolPtr {
	fn from(value: Symbol) -> Self {
		ArcSwap::from_pointee(value)
	}
}
