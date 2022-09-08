//! Trait extending `mlua::Lua` with Impure-specific behavior.

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

/// Only exists to extends `mlua::Lua` with new methods.
pub trait ImpureLua<'p> {
	/// Seeds the RNG, loads Teal into the registry; defines logging functions,
	/// `import` as a substitute for `require`, and functions required by all
	/// contexts that require no other state to be captured.
	/// If `safe` is `false`, [`mlua::prelude::LuaStdLib::DEBUG`]
	/// will be loaded into the constructed state.
	/// If `clientside` is `true`, the state's registry will contain the key-value
	/// pair `['clientside'] = true`. Otherwise, this key will be left nil.
	fn new_ex(
		safe: bool,
		clientside: bool,
		vfs: Arc<RwLock<VirtualFs>>,
	) -> Result<Lua, mlua::Error>;

	/// For guaranteeing that loaded chunks are text.
	fn safeload<'lua, 'a, S>(
		&'lua self,
		chunk: &'a S,
		name: &str,
		env: LuaTable<'lua>,
	) -> LuaChunk<'lua, 'a>
	where
		S: mlua::AsChunk<'lua> + ?Sized;

	/// Retrieve `envs.std` from the registry.
	fn env_std(&self) -> LuaTable;

	/// Adds `math`, `string`, and `table` standard libraries to an environment,
	/// as well as several standard free functions and `_VERSION`.
	fn env_init_std(&self, env: &LuaTable);
}

impl<'p> ImpureLua<'p> for mlua::Lua {
	fn new_ex(
		safe: bool,
		clientside: bool,
		vfs: Arc<RwLock<VirtualFs>>,
	) -> Result<Lua, mlua::Error> {
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

		if clientside {
			ret.set_named_registry_value("clientside", true)
				.expect("`ImpureLua::new_ex` failed to set state ID in registry.");
		}

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

		let envs = ret
			.create_table()
			.expect("`ImpureLua::new_ex`: failed to create `envs` table.");

		// Standard-only environment, used as a basis for other environments

		let stdenv = ret
			.create_table()
			.expect("`ImpureLua::new_ex`: failed to create standard environment table.");

		ret.env_init_std(&stdenv);

		match envs.set("std", stdenv) {
			Ok(()) => {}
			Err(err) => {
				error!("`ImpureLua::new_ex`: Failed to set registry `envs.std`.");
				return Err(err);
			}
		}

		match ret.set_named_registry_value("envs", envs) {
			Ok(()) => {}
			Err(err) => {
				error!("`ImpureLua::new_ex`: Failed to put `envs` table into registry.");
				return Err(err);
			}
		};

		// Load the Teal compiler into the registry

		let teal: LuaTable = match ret
			.safeload(include_str!("./teal.lua"), "teal", ret.env_std())
			.eval()
		{
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

		// Common functions: `import`

		{
			let import = ret.create_function(move |l, path: String| -> LuaResult<LuaValue> {
				let vfsg = vfs.read();

				let bytes = match vfsg.read(&path) {
					Ok(b) => b,
					Err(err) => {
						return Err(LuaError::ExternalError(Arc::new(err)));
					}
				};

				return l.safeload(bytes, path.as_str(), l.env_std()).eval();
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

	fn safeload<'lua, 'a, S>(
		&'lua self,
		chunk: &'a S,
		name: &str,
		env: LuaTable<'lua>,
	) -> LuaChunk<'lua, 'a>
	where
		S: mlua::AsChunk<'lua> + ?Sized,
	{
		self.load(chunk)
			.set_mode(mlua::ChunkMode::Text)
			.set_environment(env)
			.expect("`ImpureLua::safeload()`: Got malformed environment.")
			.set_name(name)
			.expect("`ImpureLua::safeload()`: Got unsanitised name.")
	}

	fn env_std(&self) -> LuaTable {
		let envs: LuaTable = self
			.named_registry_value("envs")
			.expect("`ImpureLua::env_std`: Failed to get registry value `envs`.");

		envs.get("std")
			.expect("`ImpureLua::env_std`: Failed to get `envs.std`.")
	}

	fn env_init_std(&self, env: &LuaTable) {
		debug_assert!(
			env.raw_len() <= 0,
			"`ImpureLua::env_init_std`: Called on a non-empty table."
		);

		let globals = self.globals();

		const GLOBAL_KEYS: [&str; 16] = [
			"_VERSION",
			// Tables
			"math",
			"string",
			"table",
			// Free functions
			"error",
			"getmetatable",
			"ipairs",
			"next",
			"pairs",
			"pcall",
			"select",
			"tonumber",
			"tostring",
			"type",
			"unpack",
			"xpcall",
		];

		for key in GLOBAL_KEYS {
			let func = globals
				.get::<&str, LuaValue>(key)
				.expect("`ImpureLua::env_init_std`: global `{}` is missing.");

			env.set(key, func).unwrap_or_else(|err| {
				panic!(
					"`ImpureLua::env_init_std`: failed to set `{}` ({}).",
					key, err
				)
			});
		}

		let debug: LuaResult<LuaTable> = globals.get("debug");

		if let Ok(d) = debug {
			env.set("debug", d)
				.expect("`ImpureLua::env_init_std`: Failed to set `debug`.");
		}
	}
}
