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

use bitflags::bitflags;
use glam::IVec2;

use crate::data::AssetId;

pub struct Vertex {
	x: f64,
	y: f64
}

pub struct LineDef {
	id: i32,
	v1: i32,
	v2: i32,
	flags: LineDefFlags,
	special: i32,
	args: [i32; 5],
	side_front: i32,
	side_back: i32
}

bitflags! {
	pub struct LineDefFlags : u32 {
		const NONE = 0;
		/// If set, line blocks things.
		const BLOCK_THINGS = 1 << 0;
		/// If set, line blocks monsters.
		const BLOCK_MONS = 1 << 1;
		/// If set, line is 2S.
		const TWO_SIDED = 1 << 2;
		/// If set, upper texture is unpegged.
		const DONT_PEG_TOP = 1 << 3;
		/// If set, lower texture is unpegged.
		const DONT_PEG_BOTTOM = 1 << 4;
		/// If set, drawn as 1S on map.
		const SECRET = 1 << 5;
		/// If set, blocks sound propagation.
		const BLOCK_SOUND = 1 << 6;
		/// If set, line is never drawn on map.
		const DONT_DRAW = 1 << 7;
		/// If set, line always appears on map.
		const MAPPED = 1 << 8;
		/// If set, linedef passes use action.
		const PASS_USE = 1 << 9;
		/// Strife translucency.
		const TRANSLUCENT = 1 << 10;
		/// Strife railing.
		const JUMPOVER = 1 << 11;
		/// Strife floater-blocker.
		const BLOCK_FLOATERS = 1 << 12;
		/// Player can cross.
		const ALLOW_PLAYER_CROSS = 1 << 13;
		/// Player can use.
		const ALLOW_PLAYER_USE = 1 << 14;
		/// Monsters can cross.
		const ALLOW_MONS_CROSS = 1 << 15;
		/// Monsters can use.
		const ALLOW_MONS_USE = 1 << 16;
		/// Projectile can activate.
		const IMPACT = 1 << 17;
		/// Player can push.
		const ALLOW_PLAYER_PUSH = 1 << 18;
		/// Monsters can push.
		const ALLOW_MONS_PUSH = 1 << 19;
		/// Projectiles can cross.
		const ALLOW_MISSILE_CROSS = 1 << 20;
		/// Repeatable special.
		const REPEAT_SPECIAL = 1 << 21;
	}
}

pub struct SideDef {
	offset: IVec2,
	tex_top: AssetId,
	tex_bottom: AssetId,
	tex_mid: AssetId,
	sector: i32
}

pub struct Sector {
	height_floor: i32,
	height_ceiling: i32,
	tex_floor: AssetId,
	tex_ceiling: AssetId,
	light_level: i32,
	special: i32,
	id: i32
}

pub struct Level {
	name: String,
	author_name: String
}
