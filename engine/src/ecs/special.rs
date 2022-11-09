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

use std::sync::Arc;

use mlua::prelude::*;
use parking_lot::RwLock;

use crate::sim::PlaySim;

/// Primarily for use by ACS behaviours. An entity won't have this component unless
/// the map specifies one of the fields within, or it gets added at runtime.
#[derive(Default, Debug, shipyard::Component)]
pub struct SpecialVars {
	pub tid: i64,
	pub special_i: [i64; 3],
	pub special_f: [f64; 2],
	pub args: [i64; 5],
}

impl LuaUserData for super::UserData<SpecialVars> {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		use crate::lua::Error;
		use shipyard::{View, ViewMut};

		fields.add_field_method_get("tid", |lua, this| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.read();
			let view = sim.world.borrow::<View<SpecialVars>>().unwrap();
			Ok((view[this.0]).tid)
		});

		fields.add_field_method_get("int", |lua, this| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.read();
			let view = sim.world.borrow::<View<SpecialVars>>().unwrap();
			Ok((view[this.0]).special_i[0])
		});

		fields.add_field_method_get("int1", |lua, this| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.read();
			let view = sim.world.borrow::<View<SpecialVars>>().unwrap();
			Ok((view[this.0]).special_i[1])
		});

		fields.add_field_method_get("int2", |lua, this| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.read();
			let view = sim.world.borrow::<View<SpecialVars>>().unwrap();
			Ok((view[this.0]).special_i[2])
		});

		fields.add_field_method_get("float1", |lua, this| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.read();
			let view = sim.world.borrow::<View<SpecialVars>>().unwrap();
			Ok((view[this.0]).special_f[0])
		});

		fields.add_field_method_get("float2", |lua, this| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.read();
			let view = sim.world.borrow::<View<SpecialVars>>().unwrap();
			Ok((view[this.0]).special_f[1])
		});

		fields.add_field_method_get("args", |lua, this| {
			let eid = this.0;
			let metatable = lua.create_table()?;

			metatable.set(
				"__index",
				lua.create_function(move |l, args: (LuaTable, usize)| {
					if !(1..=5).contains(&args.1) {
						return Err(LuaError::ExternalError(Arc::new(Error::IndexOutOfRange {
							given: args.1,
							min: 1,
							max: 5,
						})));
					}

					let sim = l.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
					let sim = sim.read();
					let view = sim.world.borrow::<View<SpecialVars>>().unwrap();
					Ok(view[eid].args[args.1])
				})?,
			)?;

			metatable.set(
				"__newindex",
				lua.create_function(move |l, args: (LuaTable, usize, i64)| {
					if !(1..=5).contains(&args.1) {
						return Err(LuaError::ExternalError(Arc::new(Error::IndexOutOfRange {
							given: args.1,
							min: 1,
							max: 5,
						})));
					}

					let sim = l.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
					let sim = sim.write();
					let mut view = sim.world.borrow::<ViewMut<SpecialVars>>().unwrap();
					view[eid].args[args.1] = args.2;

					Ok(())
				})?,
			)?;

			let ret = lua.create_table()?;
			ret.set_metatable(Some(metatable));
			Ok(ret)
		});

		// Setters /////////////////////////////////////////////////////////////

		fields.add_field_method_set("tid", |lua, this, int: LuaInteger| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.write();
			let mut view = sim.world.borrow::<ViewMut<SpecialVars>>().unwrap();

			(view[this.0]).tid = int;

			Ok(())
		});

		fields.add_field_method_set("int", |lua, this, int: LuaInteger| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.write();
			let mut view = sim.world.borrow::<ViewMut<SpecialVars>>().unwrap();

			(view[this.0]).special_i[0] = int;

			Ok(())
		});

		fields.add_field_method_set("int1", |lua, this, int: LuaInteger| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.write();
			let mut view = sim.world.borrow::<ViewMut<SpecialVars>>().unwrap();

			(view[this.0]).special_i[1] = int;

			Ok(())
		});

		fields.add_field_method_set("float1", |lua, this, float: LuaNumber| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.write();
			let mut view = sim.world.borrow::<ViewMut<SpecialVars>>().unwrap();

			(view[this.0]).special_f[0] = float;

			Ok(())
		});

		fields.add_field_method_set("float2", |lua, this, float: LuaNumber| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.write();
			let mut view = sim.world.borrow::<ViewMut<SpecialVars>>().unwrap();

			(view[this.0]).special_f[1] = float;

			Ok(())
		});

		fields.add_field_method_set("int2", |lua, this, int: LuaInteger| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.write();
			let mut view = sim.world.borrow::<ViewMut<SpecialVars>>().unwrap();

			(view[this.0]).special_i[2] = int;

			Ok(())
		});

		fields.add_field_method_set("args", |lua, this, table: LuaTable| {
			let sim = lua.app_data_ref::<Arc<RwLock<PlaySim>>>().unwrap();
			let sim = sim.write();
			let mut view = sim.world.borrow::<ViewMut<SpecialVars>>().unwrap();
			let comp = &mut view[this.0];

			for i in 0..5 {
				match table.get::<i64, i64>(i + 1) {
					Ok(int) => {
						comp.args[i as usize] = int;
					}
					Err(err) => {
						return Err(err);
					}
				}
			}

			Ok(())
		});
	}
}
