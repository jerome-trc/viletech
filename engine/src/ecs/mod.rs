//! Playsim entity components.

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

mod blueprint;
pub use blueprint::Blueprint;

use crate::data::AssetHandle;

use mlua::prelude::*;

#[derive(Debug, shipyard::Component)]
pub struct Constant {
	/// The sim tic on which this entity was spawned.
	spawned_tic: u32,
	blueprint: AssetHandle,
}

impl LuaUserData for Constant {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("spawned_tic", |_, this| Ok(this.spawned_tic));
	}
}
