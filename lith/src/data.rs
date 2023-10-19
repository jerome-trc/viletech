//! Pieces of data declared and inspected by the frontend.

use std::sync::atomic::{self, AtomicU32, AtomicUsize};

use cranelift::codegen::ir::UserExternalName;
use doomfront::rowan::{TextRange, TextSize};
use smallvec::SmallVec;

use crate::{
	arena::CPtr,
	filetree::FileIx,
	intern::NameIx,
	runtime,
	types::{CEvalIntrin, Scope},
	CEval, CEvalNative,
};

pub(crate) type SymPtr = CPtr<Symbol>;
pub(crate) type DatumPtr = CPtr<Datum>;

#[derive(Debug)]
pub(crate) struct Symbol {
	pub(crate) location: Location,
	pub(crate) datum: DatumPtr,
}

impl Drop for Symbol {
	fn drop(&mut self) {
		if let Some(ptr) = self.datum.as_ptr() {
			unsafe {
				std::ptr::drop_in_place(ptr.as_ptr());
			}
		}
	}
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
pub(crate) enum Datum {
	Function(Function),
	/// In a `* => rename` import, this is the type of `rename`.
	Container(Scope),
	Primitive(Primitive),
	SymConst(SymConst),
}

// Common details //////////////////////////////////////////////////////////////

/// "Qualified type".
#[derive(Debug)]
pub(crate) enum QualType {
	Normal {
		inner: SymPtr,
		array_dims: SmallVec<[ArrayLength; 1]>,
		optional: bool,
		reference: bool,
	},
	Any {
		optional: bool,
	},
	Type {
		array_dims: SmallVec<[ArrayLength; 1]>,
		optional: bool,
	},
}

#[derive(Debug)]
pub(crate) struct ArrayLength(AtomicUsize);

impl ArrayLength {
	#[must_use]
	pub(crate) fn get(&self) -> usize {
		let ret = self.0.load(atomic::Ordering::Acquire);
		debug_assert_ne!(ret, 0);
		ret
	}

	pub(crate) fn set(&self, len: usize) {
		debug_assert_eq!(self.0.load(atomic::Ordering::Acquire), 0);
		debug_assert_ne!(len, 0);
		self.0.store(len, atomic::Ordering::Release);
	}
}

impl Default for ArrayLength {
	fn default() -> Self {
		Self(AtomicUsize::new(0))
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

/// The "confinement" system is designed for use in games which have both a
/// single- and multi-player component and need some symbols to operate only
/// "client-side" without affecting the gameplay simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Confinement {
	None,
	Ui,
	Sim,
}

// Function ////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct Function {
	pub(crate) flags: FunctionFlags,
	pub(crate) visibility: Visibility,
	pub(crate) confine: Confinement,
	pub(crate) inlining: Inlining,
	pub(crate) params: Vec<Parameter>,
	pub(crate) return_type: QualType,
	pub(crate) code: FunctionCode,
}

#[derive(Debug)]
pub(crate) enum FunctionCode {
	/// Function was defined entirely in Lith source.
	Ir {
		/// An index into [`crate::compile::Compiler::ir`].
		ir_ix: AtomicU32,
	},
	/// Function is Rust-defined, intrinsic to the compiler.
	Builtin {
		uext_name: UserExternalName,
		rt: Option<extern "C" fn(*mut runtime::Context, ...)>,
		ceval: Option<CEvalIntrin>,
	},
	/// Function is Rust-defined, registered externally.
	Native {
		uext_name: UserExternalName,
		rt: Option<extern "C" fn(*mut runtime::Context, ...)>,
		ceval: Option<CEvalNative>,
	},
}

impl FunctionCode {
	pub(crate) const IR_IX_UNDEFINED: u32 = u32::MAX;
	pub(crate) const IR_IX_PENDING: u32 = u32::MAX - 1;
	pub(crate) const IR_IX_FAILED: u32 = u32::MAX - 2;
}

#[derive(Debug)]
pub(crate) struct Parameter {
	pub(crate) name: NameIx,
	pub(crate) qtype: QualType,
	pub(crate) consteval: bool,
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

// Primitive ///////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Primitive {
	Void,
	Bool,
	I8,
	I16,
	I32,
	I64,
	I128,
	U8,
	U16,
	U32,
	U64,
	U128,
	F32,
	F64,
}

// SymConst ////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct SymConst {
	pub(crate) visibility: Visibility,
	pub(crate) qtype: QualType,
	pub(crate) init: CEval,
}
