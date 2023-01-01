/*
** -----------------------------------------------------------------------------
** Copyright 1998-2012 Randy Heit
** All rights reserved.
**
** Redistribution and use in source and binary forms, with or without
** modification, are permitted provided that the following conditions
** are met:
**
** 1. Redistributions of source code must retain the above copyright
**    notice, this list of conditions and the following disclaimer.
** 2. Redistributions in binary form must reproduce the above copyright
**    notice, this list of conditions and the following disclaimer in the
**    documentation and/or other materials provided with the distribution.
** 3. The name of the author may not be used to endorse or promote products
**    derived from this software without specific prior written permission.
**
** THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
** IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
** OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
** IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY DIRECT, INDIRECT,
** INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
** NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
** DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
** THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
** (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
** THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
** -----------------------------------------------------------------------------
*/

use num::traits::FromPrimitive;
use num_derive::FromPrimitive;

use crate::{data::AssetHandle, ecs::EntityId, math::IRect32};

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
