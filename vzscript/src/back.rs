//! VZScript's [Cranelift](cranelift)-based backend.

use std::{
	ffi::{c_char, c_int, c_void},
	io::Cursor,
	sync::Arc,
};

use parking_lot::RwLock;

use crate::{compile::Compiler, Project, Runtime};

pub type SsaType = cranelift::codegen::ir::Type;
pub type SsaValues = smallvec::SmallVec<[SsaType; 1]>;
