//! Pieces of data declared and inspected by the frontend.

use cranelift_module::FuncId;
use doomfront::rowan::TextRange;

use crate::{
	arena::CPtr, filetree::FileIx, intern::NameIx, runtime, CEvalIntrin, CEvalNative, IrFunction,
	Scope,
};

pub(crate) type SymPtr = CPtr<Symbol>;
pub(crate) type DatumPtr = CPtr<Datum>;
pub(crate) type CodePtr = CPtr<FunctionCode>;

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
	/// The start is always the very start of a symbol's highest node.
	/// Use this to locate the part of the AST a symbol came from.
	pub(crate) span: TextRange,
	/// Index to an element in [`crate::filetree::FileTree::graph`].
	pub(crate) file_ix: FileIx,
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

#[derive(Debug)]
pub(crate) enum Datum {
	Function(Function),
	/// In a `* => rename` import, this is the type of `rename`.
	Container(Scope),
	Primitive(Primitive),
	SymConst(SymConst),
}

// Common details //////////////////////////////////////////////////////////////

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
	pub(crate) return_type: SymPtr,
	pub(crate) code: CodePtr,
}

impl Drop for Function {
	fn drop(&mut self) {
		if let Some(ptr) = self.code.as_ptr() {
			unsafe {
				std::ptr::drop_in_place(ptr.as_ptr());
			}
		}
	}
}

#[derive(Debug)]
pub(crate) enum FunctionCode {
	/// Function was defined entirely in Lith source.
	Ir { ir: IrFunction, id: FuncId },
	/// Function is Rust-defined, intrinsic to the compiler.
	Builtin {
		rt: Option<extern "C" fn(runtime::Context, ...)>,
		ceval: Option<CEvalIntrin>,
	},
	/// Function is Rust-defined, registered externally.
	Native {
		rt: Option<extern "C" fn(runtime::Context, ...)>,
		ceval: Option<CEvalNative>,
	},
}

#[derive(Debug)]
pub(crate) struct Parameter {
	pub(crate) name: NameIx,
	pub(crate) type_spec: SymPtr,
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
	pub(crate) type_spec: SymPtr,
}
