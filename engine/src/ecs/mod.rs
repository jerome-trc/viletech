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
mod special;

use std::{marker::PhantomData, sync::Arc};

use mlua::prelude::*;
use parking_lot::RwLock;
use shipyard::{Component, EntityId};

use crate::{data::AssetHandle, sim::PlaySim};

pub use blueprint::Blueprint;
pub use special::SpecialVars;

/// Userdata objects wrapping this type are stored in script-accessible entity
/// tables and used as proxies for the real ECS components.
pub struct UserData<T: Component>(EntityId, PhantomData<T>);

////////////////////////////////////////////////////////////////////////////////

/// Component for data which is baked into a newly-spawned entity and never changes.
#[derive(Debug, Component)]
pub struct Constant {
	/// The sim tic on which this entity was spawned.
	spawned_tic: u32,
	blueprint: AssetHandle,
}

impl LuaUserData for UserData<Constant> {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		use shipyard::View;

		fields.add_field_method_get("spawned_tic", |lua, this| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.read();
			let view = sim.world.borrow::<View<Constant>>().unwrap();
			Ok((view[this.0]).spawned_tic)
		});
	}
}
