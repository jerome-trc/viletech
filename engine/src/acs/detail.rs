/*

Copyright (C) 2022 ***REMOVED***

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

*/

const HUDMSG_LAYER_SHIFT: i32 = 12;
const HUDMSG_LAYER_MASK: i32 = 0x0000F000;

const HUDMSG_VIS_SHIFT: i32 = 16;
const HUDMSG_VIS_MASK: i32 = 0x00070000;

struct LocalVars(Vec<i32>);

struct LocalArrayEntry {
	size: u32,
	offset: i32,
}

#[derive(Default)]
struct LocalArray {
	entries: Vec<LocalArrayEntry>,
}

#[derive(Default)]
pub(super) struct ScriptPointer {
	number: i32,
	address: u32,
	kind: u8,
	arg_count: u8,
	var_count: u16,
	flags: u16,
	local_arrays: LocalArray,
}

impl ScriptPointer {
	pub(super) fn from_hexen(&mut self, ptrh: &ScriptPointerH) {
		self.number = (ptrh.number & 1000) as i32;
		self.kind = (ptrh.number / 1000) as u8;
		self.arg_count = (ptrh.arg_count) as u8;
		self.address = ptrh.address;
	}

	pub(super) fn from_zdoom(&mut self, ptrz: &ScriptPointerZD) {
		self.number = ptrz.number as i32;
		self.kind = ptrz.kind as u8;
		self.arg_count = ptrz.arg_count as u8;
		self.address = ptrz.address;
	}

	pub(super) fn from_intermediate(&mut self, ptri: &ScriptPointerI) {
		self.number = ptri.number as i32;
		self.kind = ptri.kind;
		self.arg_count = ptri.arg_count;
		self.address = ptri.address;
	}
}

pub(super) struct ScriptFunction {
	arg_count: u8,
	has_retval: u8,
	import_num: u8,
	local_count: i32,
	address: u32,
	local_array: LocalArray,
}

/*
https://github.com/rust-lang/rust/issues/100878

#[must_use]
pub(super) fn ascii_id(bytes: [u8; 4]) -> u32 {
	(bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24)) as u32
}
*/

const STACK_SIZE: usize = 4096;

struct Stack {
	memory: [i32; STACK_SIZE],
	pointer: usize,
}

impl Default for Stack {
	fn default() -> Self {
		Self {
			memory: [0i32; STACK_SIZE],
			pointer: 0,
		}
	}
}

// Intermediate types that match representatons in object files

/// ZDoom's intermediate script representation.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct ScriptPointerZD {
	number: i16,
	kind: u16,
	arg_count: u32,
	address: u32,
}

/// Hexen's original script representation.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct ScriptPointerH {
	// This script's kind is `number / 1000`.
	number: u32,
	address: u32,
	arg_count: u32,
}

/// ZDoom's current in-file script representation.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct ScriptPointerI {
	number: i16,
	kind: u8,
	arg_count: u8,
	address: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct ScriptFunctionFileRepr {
	arg_count: u8,
	local_count: u8,
	has_retval: u8,
	import_num: u8,
	address: u32,
}
