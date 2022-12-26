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
along with this program. If not, see <http://www.gnu.org/licenses/>.

*/

use crate::data::AssetHandle;

/// Component for data which is baked into a newly-spawned entity and never changes.
#[derive(Debug)]
pub struct Constant {
	/// The sim tic on which this entity was spawned.
	spawned_tic: u32,
	blueprint: AssetHandle,
}

/// Primarily for use by ACS behaviours. An entity won't have this component unless
/// the map specifies one of the fields within, or it gets added at runtime.
#[derive(Default, Debug)]
pub struct SpecialVars {
	pub tid: i64,
	pub special_i: [i64; 3],
	pub special_f: [f64; 2],
	pub args: [i64; 5],
}
