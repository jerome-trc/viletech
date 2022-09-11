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
	/// Seeds the RNG, defines some dependency-free global functions (logging, etc.).
	/// If `safe` is `false`, the debug and FFI libraries are loaded.
	/// If `clientside` is `true`, the state's registry will contain the key-value
	/// pair `['clientside'] = true`. Otherwise, this key will be left nil.
	fn new_ex(safe: bool, clientside: bool) -> LuaResult<Lua>;

	/// Modifies the Lua global environment to be more conducive to a safe,
	/// Impure-suitable sandbox, and adds numerous Impure-specific symbols.
	fn global_init(&self, vfs: Option<Arc<RwLock<VirtualFs>>>) -> LuaResult<()>;

	/// For guaranteeing that loaded chunks are text.
	fn safeload<'lua, 'a, S>(
		&'lua self,
		chunk: &'a S,
		name: &str,
		env: LuaTable<'lua>,
	) -> LuaChunk<'lua, 'a>
	where
		S: mlua::AsChunk<'lua> + ?Sized;

	/// Adds `math`, `string`, and `table` standard libraries to an environment,
	/// as well as several standard free functions and `_VERSION`.
	fn envbuild_std(&self, env: &LuaTable);
}

impl<'p> ImpureLua<'p> for mlua::Lua {
	fn new_ex(safe: bool, clientside: bool) -> LuaResult<Lua> {
		// Note: `io`, `os`, and `package` aren't sandbox-safe by themselves.
		// They either get pruned of dangerous functions by `global_init` or
		// are deleted now and may get returned in reduced form in the future.

		#[rustfmt::skip]
		let safe_libs =
			LuaStdLib::BIT |
			LuaStdLib::IO |
			LuaStdLib::JIT |
			LuaStdLib::MATH |
			LuaStdLib::OS |
			LuaStdLib::PACKAGE |
			LuaStdLib::STRING |
			LuaStdLib::TABLE;

		let ret = if let true = safe {
			Lua::new_with(safe_libs, LuaOptions::default())?
		} else {
			unsafe {
				Lua::unsafe_new_with(
					safe_libs | LuaStdLib::DEBUG | LuaStdLib::FFI,
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

		let impure = match ret.create_table() {
			Ok(t) => t,
			Err(err) => {
				error!("Failed to create global table `impure`.");
				return Err(err);
			}
		};

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

	fn envbuild_std(&self, env: &LuaTable) {
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

	fn global_init(&self, vfs: Option<Arc<RwLock<VirtualFs>>>) -> LuaResult<()> {
		let globals = self.globals();

		// Many functions (e.g. `jit`, `setfenv`) aren't deleted here,
		// but aren't included in any user-facing environment

		const KEYS_STD_GLOBAL: [&str; 5] = [
			"io",
			"package",
			// Free functions
			"collectgarbage",
			"module",
			"print",
		];

		for key in KEYS_STD_GLOBAL {
			globals.set(key, LuaValue::Nil)?;
		}

		// Delete unsafe OS standard library functions /////////////////////////

		const KEYS_STD_OS: [&str; 7] = [
			"execute",
			"exit",
			"getenv",
			"remove",
			"rename",
			"setlocale",
			"tmpname",
		];

		let g_os: LuaTable = globals.get("os")?;

		for key in KEYS_STD_OS {
			g_os.set(key, LuaValue::Nil)?;
		}

		// Teal compiler ///////////////////////////////////////////////////////

		let teal: LuaTable = self
			.load(include_str!("./teal.lua"))
			.set_mode(mlua::ChunkMode::Text)
			.set_name("teal")?
			.eval()?;

		globals.set("teal", teal)?;

		// Add virtual FS API, if applicable ///////////////////////////////////

		if let Some(vfs) = vfs {
			let v = vfs.clone();

			let import = self.create_function(move |l, path: String| -> LuaResult<LuaValue> {
				let vfsg = v.read();

				let bytes = match vfsg.read(&path) {
					Ok(b) => b,
					Err(err) => {
						return Err(LuaError::ExternalError(Arc::new(err)));
					}
				};

				let string = match std::str::from_utf8(bytes) {
					Ok(s) => s,
					Err(err) => {
						return Err(LuaError::ExternalError(Arc::new(err)));
					}
				};

				let teal: LuaTable = l
					.named_registry_value("teal")
					.expect("Teal compiler wasn't loaded into this state.");
				let teal: LuaFunction = teal.get("gen").expect("Teal compiler is malformed.");

				let chunk = match teal.call::<&str, String>(string) {
					Ok(s) => s,
					Err(err) => {
						return Err(LuaError::ExternalError(Arc::new(err)));
					}
				};

				let env = l
					.globals()
					.call_function("getenv", 0)
					.expect("`import` failed to retrieve the current environment.");

				return l.safeload(&chunk, path.as_str(), env).eval();
			});

			match import {
				Ok(i) => {
					self.globals().set("import", i)?;
				}
				Err(err) => {
					return Err(err);
				}
			}

			let g_vfs = self.create_table()?;

			let g_vfs_read = self.create_function(move |l, path: String| {
				let guard = vfs.read();

				let handle = match guard.lookup(&path) {
					Some(h) => h,
					None => {
						return Ok(LuaValue::Nil);
					}
				};

				let content = match handle.read_str() {
					Ok(s) => s,
					Err(err) => {
						error!(
							"File contents are invalid UTF-8: {}
							Error: {}",
							path, err
						);
						return Ok(LuaValue::Nil);
					}
				};

				let string = match l.create_string(content) {
					Ok(s) => s,
					Err(err) => {
						return Err(err);
					}
				};

				Ok(LuaValue::String(string))
			})?;

			g_vfs.set("read", g_vfs_read)?;
			globals.set("vfs", g_vfs)?;
		}

		Ok(())
	}
}
