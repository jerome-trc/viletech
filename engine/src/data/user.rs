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

use std::collections::HashMap;

use bitflags::bitflags;

use crate::gfx::Rgb32;

bitflags! {
	pub struct PrefFlags: u8 {
		const NONE = 0;
		/// If unset, this pref only applies client-side.
		const SIM = 1 << 0;
	}
}

/// The second value holds the default.
pub enum PrefKind {
	Bool(bool, bool),
	Int(i32, i32),
	Float(f32, f32),
	Color(Rgb32, Rgb32),
	String(String, String),
}

pub enum UserGender {
	Female = 0,
	Male = 1,
	Neutral = 2,
	Object = 3,
}

pub struct UserPref {
	kind: PrefKind,
	flags: PrefFlags,
}

pub struct UserProfile {
	name: String,
	prefs: HashMap<String, UserPref>,
}
