//! "Symbol" data structures.
//! Pieces of data declared and inspected by the frontend.

use cranelift::{
	codegen::{data_value::DataValue, ir::UserExternalName},
	prelude::Variable,
};
use doomfront::rowan::{TextRange, TextSize};
use smallvec::SmallVec;
use util::pushvec::PushVec;

use crate::{
	compile::{intern::NameIx, NativeFunc},
	filetree::FileIx,
	types::{Scope, TypeNPtr, TypePtr},
};

#[derive(Debug)]
pub struct Symbol {
	pub(crate) location: Location,
	pub(crate) datum: SymDatum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Location {
	/// Index to an element in [`crate::filetree::FileTree::graph`].
	pub(crate) file_ix: FileIx,
	/// The start is always the very start of a symbol's highest node.
	/// Use this to locate the part of the AST a symbol came from.
	pub(crate) span: TextRange,
}

impl Location {
	#[must_use]
	pub(crate) fn full_file(file_ix: FileIx) -> Self {
		Self {
			span: TextRange::new(0.into(), 0.into()),
			file_ix,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SymbolId {
	pub(crate) file_ix: FileIx,
	pub(crate) offs: TextSize,
}

impl SymbolId {
	#[must_use]
	pub(crate) fn new(location: Location) -> Self {
		Self {
			file_ix: location.file_ix,
			offs: location.span.start(),
		}
	}
}

impl From<SymbolId> for UserExternalName {
	fn from(value: SymbolId) -> Self {
		Self {
			namespace: value.file_ix.index() as u32,
			index: value.offs.into(),
		}
	}
}

impl From<UserExternalName> for SymbolId {
	fn from(value: UserExternalName) -> Self {
		Self {
			file_ix: FileIx::new(value.namespace as usize),
			offs: TextSize::from(value.index),
		}
	}
}

#[derive(Debug)]
pub(crate) enum SymDatum {
	/// In a `* => rename` import, this is the type of `rename`.
	Container(FileIx, Scope),
	Function(Function),
	Local(LocalVar),
	SymConst(SymConst),
}

// Common details //////////////////////////////////////////////////////////////

/// The "confinement" system is designed for use in games which have both a
/// single- and multi-player component and need some symbols to operate only
/// "client-side" without affecting the gameplay simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Confinement {
	None,
	Ui,
	Sim,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Visibility {
	/// Visible to all libraries.
	/// Corresponds to the `public` keyword.
	Export,
	/// Visible only to declaring library.
	/// Corresponds to the absence of a visibility specifier.
	#[default]
	Default,
	/// Visible within container only.
	/// Corresponds to the `private` keyword.
	Hidden,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum TypeSpec {
	Normal(TypeNPtr),
	/// Corresponds to `: type_t`.
	Type,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // TODO: revisit.
pub(crate) enum ConstInit {
	Type(TypeNPtr),
	Value(PushVec<DataValue>),
}

// Function ////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Function {
	pub(crate) flags: FunctionFlags,
	pub(crate) _visibility: Visibility,
	pub(crate) confine: Confinement,
	pub(crate) inlining: Inlining,
	pub(crate) params: Vec<Parameter>,
	pub(crate) ret_type: TypeSpec,
	pub(crate) kind: FunctionKind,
}

impl Function {
	#[must_use]
	pub(crate) fn signature_incomplete(&self) -> bool {
		if let TypeSpec::Normal(type_ptr) = &self.ret_type {
			return type_ptr.as_ptr().is_none();
		};

		for param in &self.params {
			if let ParamType::Normal(type_ptr) = &param.ptype {
				return type_ptr.as_ptr().is_none();
			} else if let Some(ConstInit::Type(type_ptr)) = &param.default {
				return type_ptr.as_ptr().is_none();
			} else {
				unreachable!()
			}
		}

		false
	}
}

#[derive(Debug)]
pub(crate) enum FunctionKind {
	/// Function was defined entirely in Lith source.
	Ir,
	/// Function is Rust-defined and either intrinsic to the compiler
	/// or registered externally by the embedder.
	Internal {
		uext_name: UserExternalName,
		inner: NativeFunc,
	},
}

unsafe impl Send for FunctionKind {}
unsafe impl Sync for FunctionKind {}

#[derive(Debug)]
pub(crate) enum ParamType {
	Normal(TypeNPtr),
	/// Corresponds to `: any_t`.
	Any,
	/// Corresponds to `: type_t`.
	Type,
}

#[derive(Debug)]
pub(crate) struct Parameter {
	pub(crate) name: NameIx,
	pub(crate) ptype: ParamType,
	pub(crate) consteval: bool,
	pub(crate) reference: ParamRef,
	pub(crate) default: Option<ConstInit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub(crate) enum ParamRef {
	None,
	Immutable,
	Mutable,
}

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub(crate) struct FunctionFlags: u8 {
		/// An annotation has hinted that the function is unlikely to be called.
		const COLD = 1 << 0;
		/// If the return type is not `void`, do not emit a warning if this
		/// function is called without explicit consumption of its return value.
		const CAN_DISCARD = 1 << 1;
	}
}

#[derive(Debug, Default)]
pub(crate) enum Inlining {
	/// Corresponds to the annotation `#[inline(never)]`.
	Never,
	#[default]
	Normal,
	/// Corresponds to the annotation `#[inline]`.
	More,
	/// Corresponds to the annotation `#[inline(extra)]`.
	Extra,
}

// LocalVar ////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct LocalVar {
	pub(crate) abi_vars: SmallVec<[Variable; 1]>,
	pub(crate) mutable: bool,
	pub(crate) tspec: TypePtr,
}

// SymConst ////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct SymConst {
	pub(crate) _visibility: Visibility,
	pub(crate) tspec: TypeSpec,
	pub(crate) init: ConstInit,
}
