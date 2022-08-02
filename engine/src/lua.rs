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

use crate::vfs::VirtualFs;
use log::{debug, error, info, warn};
use mlua::prelude::*;
use parking_lot::RwLock;
use std::{
	sync::Arc,
	time::{SystemTime, UNIX_EPOCH},
};

pub trait ImpureLua<'p> {
	fn new_ex(safe: bool, vfs: Arc<RwLock<VirtualFs>>) -> Result<Lua, mlua::Error>;
	fn safeload<'lua, 'a, S>(&'lua self, chunk: &'a S, name: &str) -> LuaChunk<'lua, 'a>
	where
		S: mlua::AsChunk<'lua> + ?Sized;
}

impl<'p> ImpureLua<'p> for mlua::Lua {
	fn new_ex(safe: bool, vfs: Arc<RwLock<VirtualFs>>) -> Result<Lua, mlua::Error> {
		let ret = if let true = safe {
			Lua::new_with(
				LuaStdLib::JIT
					| LuaStdLib::STRING | LuaStdLib::BIT
					| LuaStdLib::MATH | LuaStdLib::TABLE,
				LuaOptions::default(),
			)?
		} else {
			unsafe {
				Lua::unsafe_new_with(
					LuaStdLib::JIT
						| LuaStdLib::STRING | LuaStdLib::BIT
						| LuaStdLib::MATH | LuaStdLib::TABLE
						| LuaStdLib::DEBUG,
					LuaOptions::default(),
				)
			}
		};

		// Seed the Lua's random state for trivial (i.e. client-side) purposes

		{
			let rseed: LuaFunction = ret
				.globals()
				.get::<_, LuaTable>("math")?
				.get::<_, LuaFunction>("randomseed")?;
			let seed = SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.expect("Failed to retrieve system time.")
				.as_millis() as u32;
			match rseed.call::<u32, ()>(seed) {
				Ok(()) => {}
				Err(err) => warn!("Failed to seed a Lua state's RNG: {}", err),
			};
		}

		// Load the Teal compiler into the registry

		let teal: LuaTable = match ret.safeload(include_str!("./teal.lua"), "teal").eval() {
			Ok(t) => t,
			Err(err) => {
				return Err(err);
			}
		};

		match ret.set_named_registry_value("teal", teal) {
			Ok(()) => {}
			Err(err) => {
				return Err(err);
			}
		};

		let impure = match ret.create_table() {
			Ok(t) => t,
			Err(err) => {
				error!("Failed to create global table `impure`.");
				return Err(err);
			}
		};

		// Common functions: logging

		impure.set(
			"log",
			ret.create_function(|_, msg: String| {
				info!("{}", msg);
				Ok(())
			})?,
		)?;

		impure.set(
			"warn",
			ret.create_function(|_, msg: String| {
				warn!("{}", msg);
				Ok(())
			})?,
		)?;

		impure.set(
			"err",
			ret.create_function(|_, msg: String| {
				error!("{}", msg);
				Ok(())
			})?,
		)?;

		impure.set(
			"debug",
			ret.create_function(|_, msg: String| {
				debug!("{}", msg);
				Ok(())
			})?,
		)?;

		// Common functions: `import()`

		{
			let import = ret.create_function(move |l, path: String| -> LuaResult<LuaValue> {
				let vfsg = vfs.read();

				let bytes = match vfsg.read(&path) {
					Ok(b) => b,
					Err(err) => {
						return Err(LuaError::ExternalError(Arc::new(err)));
					}
				};

				return l.safeload(bytes, path.as_str()).eval();
			});

			match import {
				Ok(i) => {
					ret.globals().set("import", i)?;
				}
				Err(err) => {
					error!("Failed to initialise Lua function: `import()`.");
					return Err(err);
				}
			}
		}

		// Common functions: engine information

		impure.set(
			"version",
			ret.create_function(|_, _: ()| {
				Ok((
					env!("CARGO_PKG_VERSION_MAJOR"),
					env!("CARGO_PKG_VERSION_MINOR"),
					env!("CARGO_PKG_VERSION_PATCH"),
				))
			})?,
		)?;

		ret.globals().set("impure", impure)?;
		Ok(ret)
	}

	fn safeload<'lua, 'a, S>(&'lua self, chunk: &'a S, name: &str) -> LuaChunk<'lua, 'a>
	where
		S: mlua::AsChunk<'lua> + ?Sized,
	{
		self.load(chunk)
			.set_mode(mlua::ChunkMode::Text)
			.set_name(name)
			.expect("A name was not sanitised before being passed to `ImpureLua::safeload()`.")
	}
}
