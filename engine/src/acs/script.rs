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

use num::traits::FromPrimitive;
use num_derive::FromPrimitive;
use shipyard::EntityId;

use crate::{data::AssetHandle, math::IRect32};

use super::detail::{LocalArray, ScriptPointerH, ScriptPointerI, ScriptPointerZD};

#[repr(u8)]
#[derive(Default, FromPrimitive)]
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

bitflags::bitflags! {
	#[derive(Default)]
	pub(super) struct PointerFlags: u16 {
		const NET_SAFE = 1 << 0;
	}
}

#[derive(Default)]
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
		self.kind = FromPrimitive::from_u8((ptrh.number / 1000) as u8)
			.expect("An intermediate ACS script pointer type wasn't pre-validated.");
		self.arg_count = (ptrh.arg_count) as u8;
		self.address = ptrh.address;
	}

	pub(super) fn mimic_zdoom(&mut self, ptrz: &ScriptPointerZD) {
		self.number = ptrz.number as i32;
		self.kind = FromPrimitive::from_u8(ptrz.kind as u8)
			.expect("An intermediate ACS script pointer type wasn't pre-validated.");
		self.arg_count = ptrz.arg_count as u8;
		self.address = ptrz.address;
	}

	pub(super) fn mimic_intermediate(&mut self, ptri: &ScriptPointerI) {
		self.number = ptri.number as i32;
		self.kind = FromPrimitive::from_u8(ptri.kind)
			.expect("An intermediate ACS script pointer type wasn't pre-validated.");
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

#[derive(PartialEq, Eq)]
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

pub(super) struct Script {
	status: Status,

	number: i32,
	module_number: i32,
	local_vars: Vec<i32>,
	activator: EntityId,
	line: Option<usize>,
	font: Option<AssetHandle>,

	hud_height: u32,
	hud_width: u32,
	clip: IRect32,
	wrap_width: i32,

	string_builder_stack: Vec<String>,
}
