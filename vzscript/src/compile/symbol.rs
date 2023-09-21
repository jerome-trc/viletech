//! Information used by the semantic mid-section but not the backend.

use std::any::Any;

use crossbeam::atomic::AtomicCell;
use doomfront::rowan::{GreenNode, TextRange, TextSize};
use smallvec::SmallVec;
use triomphe::Arc;

use crate::{
	ast,
	back::AbiTypes,
	rti,
	sema::{CEval, CEvalVec},
	tsys::{
		ClassType, EnumType, FuncType, PrimitiveType, StructType, TypeDef, TypeHandle, UnionType,
	},
	vir, ArcSwap, ZName,
};

use super::{
	intern::{NameIx, SymbolIx},
	CEvalBuiltin, Compiler, Scope,
};

#[derive(Debug)]
pub(crate) struct Symbol {
	pub(crate) name: NameIx,
	pub(crate) location: Location,
	/// Determines whether name lookups need to happen through all namespaces
	/// (to imitate the reference implementation's global namespace) or just
	/// the library's own.
	pub(crate) zscript: bool,
	/// Sema. treats this as a sort of lock over `def`, since the only other ways
	/// for its "require" mechanism to properly wait would be to:
	/// - Perform a load, check for non-defined and non-pending, and then
	/// start an RCU, opening the way to data races, or
	/// - Start an RCU and early-out if the symbol is defined or pending,
	/// which is massively slower than a compare-swap on an integer value
	pub(crate) status: AtomicCell<DefStatus>,
	pub(crate) def: ArcSwap<Definition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Location {
	/// The start is always the very start of a symbol's highest node.
	/// Use this to locate the part of the AST a symbol came from.
	pub(crate) span: TextRange,
	/// Combine with [`Self::span`]'s start for reporting issues.
	pub(crate) short_end: TextSize,
	/// Index to an element in [`crate::compile::Compiler::sources`].
	pub(crate) lib_ix: u16,
	/// Index to an element in [`crate::IncludeTree::files`].
	pub(crate) file_ix: u16,
}

/// See [`Symbol::status`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DefStatus {
	None,
	Pending,
	Err,
	Ok,
}

impl DefStatus {
	#[allow(unused)]
	const STATIC_ASSERT_LOCK_FREE: () = {
		assert!(AtomicCell::<Self>::is_lock_free());
	};
}

impl From<&Result<Arc<Definition>, ()>> for DefStatus {
	fn from(value: &Result<Arc<Definition>, ()>) -> Self {
		match value {
			Ok(_) => Self::Ok,
			Err(_) => Self::Err,
		}
	}
}

#[derive(Debug)]
pub(crate) struct Definition {
	pub(crate) kind: DefKind,
	pub(crate) scope: Scope,
}

#[derive(Debug)]
pub(crate) enum DefKind {
	None {
		kind: Undefined,
		qname: ZName,
	},
	/// An index into [`Compiler::symbols`].
	Rename(SymbolIx),
	/// An index into [`Compiler::namespaces`].
	Import(u16),
	Pending,
	Error,

	Builtin {
		function: CEvalBuiltin,
	},
	Class {
		typedef: rti::Record,
		handle: TypeHandle<ClassType>,
	},
	Enum {
		typedef: rti::Record,
		handle: TypeHandle<EnumType>,
	},
	Field {
		typedef: rti::Handle<TypeDef>,
	},
	Function {
		typedef: rti::Record,
		handle: TypeHandle<FuncType>,
		code: FunctionCode,
	},
	Mixin {},
	Primitive {
		typedef: rti::Record,
		handle: TypeHandle<PrimitiveType>,
	},
	Struct {
		typedef: rti::Record,
		handle: TypeHandle<StructType>,
	},
	Union {
		typedef: rti::Record,
		handle: TypeHandle<UnionType>,
	},
	Value(CEval),
}

impl DefKind {
	#[must_use]
	pub(crate) fn user_facing_name(&self) -> &'static str {
		match self {
			Self::None { kind, .. } => match kind {
				Undefined::Class => "class",
				Undefined::Enum => "enum",
				Undefined::Field => "field",
				Undefined::FlagDef => "flagDef",
				Undefined::Function => "function",
				Undefined::Property => "property",
				Undefined::Mixin => "mixin",
				Undefined::Struct => "struct",
				Undefined::Union => "union",
				Undefined::Value => "constant",
			},
			Self::Rename(_) => "type alias",
			Self::Import(_) => "import",
			Self::Builtin { .. } => "builtin",
			Self::Class { .. } => "class",
			Self::Enum { .. } => "enum",
			Self::Field { .. } => "field",
			Self::Function { .. } => "function",
			Self::Mixin {} => "mixin",
			Self::Primitive { .. } => "primitive type",
			Self::Struct { .. } => "struct",
			Self::Union { .. } => "union",
			Self::Value { .. } => "constant",
			Self::Error | Self::Pending => unreachable!(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Undefined {
	Class,
	Enum,
	Field,
	FlagDef,
	Function,
	Property,
	Mixin,
	Struct,
	Union,
	Value,
}

#[derive(Debug)]
pub(crate) enum FunctionCode {
	Builtin(fn(AbiTypes) -> AbiTypes),
	Native(&'static str),
	Ir(vir::Function),
}
