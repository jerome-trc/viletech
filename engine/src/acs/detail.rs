//! Symbols that do not belong anywhere else.
//!
//! Assume all code within originates from GZDoom-original source.

const HUDMSG_LAYER_SHIFT: i32 = 12;
const HUDMSG_LAYER_MASK: i32 = 0x0000F000;

const HUDMSG_VIS_SHIFT: i32 = 16;
const HUDMSG_VIS_MASK: i32 = 0x00070000;

#[derive(Debug)]
pub(super) struct LocalVars(Vec<i32>);

#[derive(Debug)]
pub(super) struct LocalArrayEntry {
	pub(super) size: u32,
	pub(super) offset: i32,
}

#[derive(Debug, Default)]
pub(super) struct LocalArray {
	pub(super) entries: Vec<LocalArrayEntry>,
}

#[derive(Debug)]
pub(super) struct ScriptFunction {
	arg_count: u8,
	has_retval: u8,
	import_num: u8,
	local_count: i32,
	address: u32,
	local_array: LocalArray,
}

#[must_use]
pub(super) const fn ascii_id(bytes: [u8; 4]) -> u32 {
	let bytes = [
		bytes[0] as u32,
		(bytes[1] as u32) << 8,
		(bytes[2] as u32) << 16,
		(bytes[3] as u32) << 24,
	];

	bytes[0] | bytes[1] | bytes[2] | bytes[3]
}

#[derive(Debug)]
struct Stack {
	memory: [i32; Self::SIZE],
	pointer: usize,
}

impl Stack {
	const SIZE: usize = 4096;
}

impl Default for Stack {
	fn default() -> Self {
		Self {
			memory: [0i32; Self::SIZE],
			pointer: 0,
		}
	}
}

// Intermediate types that match representatons in object files

/// ZDoom's intermediate script representation.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct ScriptPointerZD {
	pub(super) number: i16,
	pub(super) kind: u16,
	pub(super) arg_count: u32,
	pub(super) address: u32,
}

/// Hexen's original script representation.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct ScriptPointerH {
	// This script's kind is `number / 1000`.
	pub(super) number: u32,
	pub(super) address: u32,
	pub(super) arg_count: u32,
}

/// ZDoom's current in-file script representation.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct ScriptPointerI {
	pub(super) number: i16,
	pub(super) kind: u8,
	pub(super) arg_count: u8,
	pub(super) address: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct ScriptFunctionFileRepr {
	pub(super) arg_count: u8,
	pub(super) local_count: u8,
	pub(super) has_retval: u8,
	pub(super) import_num: u8,
	pub(super) address: u32,
}
