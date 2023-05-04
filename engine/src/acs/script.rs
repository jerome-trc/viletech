//! An atom of ACS functionality.
//!
//! Assume all code within originates from GZDoom-original source.

use crate::{math::IRect32, sim::actor::Actor};

use super::detail::{LocalArray, ScriptPointerH, ScriptPointerI, ScriptPointerZD};

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Kind {
	Closed,
	Open,
	Respawn,
	Death,
	Enter,
	Pickup,
	BlueReturn,
	RedReturn,
	WhiteReturn,
	Lightning = 12,
	Unloading,
	Disconnect,
	Return,
	Event,
	Kill,
	Reopen,
	#[default]
	Unknown = u8::MAX,
}

impl From<u8> for Kind {
	fn from(value: u8) -> Self {
		const MAX: u8 = Kind::Reopen as u8;

		if value > MAX {
			Self::Unknown
		} else {
			unsafe { std::mem::transmute::<_, _>(value) }
		}
	}
}

bitflags::bitflags! {
	#[derive(Default)]
	pub(super) struct PointerFlags: u16 {
		const NET_SAFE = 1 << 0;
	}
}

#[derive(Debug, Default)]
pub(super) struct Pointer {
	number: i32,
	address: u32,
	kind: Kind,
	arg_count: u8,
	var_count: u16,
	flags: PointerFlags,
	local_arrays: LocalArray,
}

impl Pointer {
	pub(super) fn mimic_hexen(&mut self, ptrh: &ScriptPointerH) {
		self.number = (ptrh.number & 1000) as i32;
		// self.kind = FromPrimitive::from_u8((ptrh.number / 1000) as u8)
		// 	.expect("An intermediate ACS script pointer type wasn't pre-validated.");
		self.kind = Kind::from((ptrh.number / 1000) as u8);
		self.arg_count = (ptrh.arg_count) as u8;
		self.address = ptrh.address;
	}

	pub(super) fn mimic_zdoom(&mut self, ptrz: &ScriptPointerZD) {
		self.number = ptrz.number as i32;
		self.kind = Kind::from(ptrz.kind as u8);
		self.arg_count = ptrz.arg_count as u8;
		self.address = ptrz.address;
	}

	pub(super) fn mimic_intermediate(&mut self, ptri: &ScriptPointerI) {
		self.number = ptri.number as i32;
		self.kind = Kind::from(ptri.kind);
		self.arg_count = ptri.arg_count;
		self.address = ptri.address;
	}
}

bitflags::bitflags! {
	pub struct Flags : u8 {
		const BACKSIDE = 1 << 0;
		const HANDLE_ASPECT = 1 << 1;
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Status {
	Running,
	Suspended,
	Delayed { clock: i32 },
	TagWait { tag: i32 },
	PolyWait { polyobj: usize },
	WaitPre { key: i32 },
	Wait { key: i32 },
	ToRemove,
	DivideByZero,
	ModulusByZero,
}

#[derive(Debug)]
pub(super) struct Script {
	status: Status,

	number: i32,
	module_number: i32,
	local_vars: Vec<i32>,
	activator: Actor,
	line: Option<usize>,
	// Q: Why did GZDoom's counterpart to this type store a font?
	hud_height: u32,
	hud_width: u32,
	clip: IRect32,
	wrap_width: i32,

	string_builder_stack: Vec<String>,
}
