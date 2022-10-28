//! Trait extending [`mlua::Lua`] with Impure-specific behavior.

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
	fmt,
	sync::Arc,
	time::{SystemTime, UNIX_EPOCH},
};

/// Only exists to extends [`mlua::Lua`] with new methods.
pub trait ImpureLua<'p> {
	/// Seeds the RNG, defines some dependency-free global functions (logging, etc.).
	/// If `safe` is `false`, the debug and FFI libraries are loaded.
	/// If `clientside` is `true`, the state's registry will contain the key-value
	/// pair `['clientside'] = true`. Otherwise, this key will be left nil.
	fn new_ex(safe: bool, clientside: bool) -> LuaResult<Lua>;

	/// Modifies the Lua global environment to be more conducive to a safe,
	/// Impure-suitable sandbox, and adds numerous Impure-specific symbols.
	fn global_init(&self, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<()>;

	/// Adds `math`, `string`, and `table` standard libraries to an environment,
	/// as well as several standard free functions and `_VERSION`.
	fn envbuild_std(&self, env: &LuaTable);

	fn getfenv(&self) -> LuaTable;

	fn metatable_readonly(&self) -> LuaTable;

	/// For guaranteeing that loaded chunks are text.
	fn safeload<'lua, 'a, S>(
		&'lua self,
		chunk: &'a S,
		name: &str,
		env: LuaTable<'lua>,
	) -> LuaChunk<'lua, 'a>
	where
		S: mlua::AsChunk<'lua> + ?Sized;

	/// Generate a human-friendly string representation of
	/// any kind of Lua value via the Serpent library.
	fn repr(&self, val: LuaValue) -> LuaResult<String>;
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

		// Table for memoizing imported modules

		ret.set_named_registry_value("modules", ret.create_table()?)?;

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

		// Metatables kept in the registry

		let metas = ret
			.create_table()
			.expect("Failed to create in-registry table `metas`.");

		let readonly = ret
			.create_table()
			.expect("Failed to create meta-table `readonly`.");

		readonly
			.set(
				"__newindex",
				ret.create_function(|_, _: LuaValue| -> LuaResult<()> {
					Err(LuaError::ExternalError(Arc::new(NewIndexError)))
				})
				.expect("Failed to create `__newindex` function for `metas.readonly`."),
			)
			.expect("Failed to set function `metas.readonly.__newindex`.");

		metas
			.set("readonly", readonly)
			.expect("Failed to set `metas.readonly`.");

		ret.set_named_registry_value("metas", metas)
			.expect("Failed to set table `metas` in registry.");

		// Dependency-free Impure-specific global functions

		let impure = match ret.create_table() {
			Ok(t) => t,
			Err(err) => {
				error!("Failed to create global table `impure`.");
				return Err(err);
			}
		};

		fn log(lua: &Lua, args: LuaMultiValue, func_name: &'static str) -> Result<String, String> {
			let mut args = args.iter();

			let template = match args.next() {
				Some(val) => {
					if let LuaValue::String(s) = val {
						s
					} else {
						return Err(format!(
							"`{}` expected a format string for argument 1, but got: {:#?}",
							func_name, val
						));
					}
				}
				None => {
					return Err(format!(
						"`{}` expected at least 1 argument, but got 0.",
						func_name
					));
				}
			};

			let mut template = match formatx::Template::new(template.to_string_lossy()) {
				Ok(t) => t,
				Err(err) => {
					return Err(format!(
						"Invalid template string given to `{}`.
							Error: {}",
						func_name, err
					));
				}
			};

			for arg in args {
				match lua.repr(arg.clone()) {
					Ok(s) => template.replace_positional(s),
					Err(err) => {
						return Err(format!(
							"Formatting error in `{}` arguments: {}",
							func_name, err
						));
					}
				};
			}

			let output = match template.text() {
				Ok(s) => s,
				Err(err) => {
					return Err(format!(
						"Formatting error in `{}` arguments: {}",
						func_name, err
					));
				}
			};

			Ok(output)
		}

		impure.set(
			"log",
			ret.create_function(|lua, args: LuaMultiValue| {
				match log(lua, args, "log") {
					Ok(s) => info!("{}", s),
					Err(s) => error!("{}", s),
				};

				Ok(())
			})?,
		)?;

		impure.set(
			"warn",
			ret.create_function(|lua, args: LuaMultiValue| {
				match log(lua, args, "warn") {
					Ok(s) => warn!("{}", s),
					Err(s) => error!("{}", s),
				};

				Ok(())
			})?,
		)?;

		impure.set(
			"err",
			ret.create_function(|lua, args: LuaMultiValue| {
				match log(lua, args, "err") {
					Ok(s) => error!("{}", s),
					Err(s) => error!("{}", s),
				};

				Ok(())
			})?,
		)?;

		impure.set(
			"debug",
			ret.create_function(|lua, args: LuaMultiValue| {
				match log(lua, args, "debug") {
					Ok(s) => debug!("{}", s),
					Err(s) => error!("{}", s),
				};

				Ok(())
			})?,
		)?;

		impure.set(
			"version",
			ret.create_function(|_, _: ()| {
				Ok((
					env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap(),
					env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap(),
					env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap(),
				))
			})?,
		)?;

		ret.globals().set("impure", impure)?;

		Ok(ret)
	}

	fn global_init(&self, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<()> {
		fn delete_g(globals: &LuaTable) -> LuaResult<()> {
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

			Ok(())
		}

		fn delete_g_os(globals: &LuaTable) -> LuaResult<()> {
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

			Ok(())
		}

		fn g_import(lua: &Lua, globals: &LuaTable, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<()> {
			globals.set(
				"import",
				lua.create_function(move |l, path: String| -> LuaResult<LuaValue> {
					let loaded: LuaTable = l
						.named_registry_value("modules")
						.expect("Registry sub-table `modules` wasn't pre-initialised.");

					match loaded.get::<&str, LuaValue>(&path) {
						Ok(module) => {
							if let LuaValue::Nil = module {
								// It wasn't found. Proceed with VFS read
							} else {
								return Ok(module);
							}
						}
						Err(err) => {
							error!("Failed to import Lua module from path: {}", path);
							return Err(err);
						}
					}

					let vfs = vfs.read();

					let bytes = match vfs.read(&path) {
						Ok(b) => b,
						Err(err) => {
							return Err(LuaError::ExternalError(Arc::new(err)));
						}
					};

					let chunk = match std::str::from_utf8(bytes) {
						Ok(s) => s,
						Err(err) => {
							return Err(LuaError::ExternalError(Arc::new(err)));
						}
					};

					match l
						.safeload(chunk, path.as_str(), l.getfenv())
						.eval::<LuaValue>()
					{
						Ok(ret) => {
							loaded.set::<&str, LuaValue>(&path, ret.clone())?;
							Ok(ret)
						}
						Err(err) => {
							error!("{}", err);
							Ok(LuaValue::Nil)
						}
					}
				})?,
			)
		}

		fn g_vfs_read(lua: &Lua, g_vfs: &LuaTable, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<()> {
			let func = lua.create_function(move |l, path: String| {
				let vfs = vfs.read();

				let handle = match vfs.lookup(&path) {
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

			g_vfs.set("read", func)
		}

		let globals = self.globals();
		let g_vfs = self.create_table()?;

		delete_g(&globals)?;
		delete_g_os(&globals)?;
		g_import(self, &globals, vfs.clone())?;
		g_vfs_read(self, &g_vfs, vfs.clone())?;

		globals.set("vfs", g_vfs)?;

		// Script "prelude" ////////////////////////////////////////////////////

		let vfsg = vfs.read();

		let utils = vfsg
			.read_str("/impure/lua/utils.lua")
			.map_err(|err| LuaError::ExternalError(Arc::new(err)))?;
		let utils = self.safeload(utils, "utils", globals.clone());
		utils.eval()?;

		let array = vfsg
			.read_str("/impure/lua/array.lua")
			.map_err(|err| LuaError::ExternalError(Arc::new(err)))?;
		let array = self.safeload(array, "array", globals.clone());
		let array: LuaTable = array.eval()?;
		globals.set("array", array)?;

		let map = vfsg
			.read_str("/impure/lua/map.lua")
			.map_err(|err| LuaError::ExternalError(Arc::new(err)))?;
		let map = self.safeload(map, "map", globals.clone());
		let map: LuaTable = map.eval()?;
		globals.set("map", map)?;

		drop(vfsg);

		Ok(())
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

	fn getfenv(&self) -> LuaTable {
		self.globals()
			.call_function("getfenv", ())
			.expect("Failed to retrieve the current environment.")
	}

	fn metatable_readonly(&self) -> LuaTable {
		self.named_registry_value::<_, LuaTable>("metas")
			.unwrap()
			.get("readonly")
			.unwrap()
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

	fn repr(&self, val: LuaValue) -> LuaResult<String> {
		self.globals().call_function::<_, _, String>("repr", val)
	}
}

#[derive(Debug)]
pub struct NewIndexError;

impl std::error::Error for NewIndexError {}

impl fmt::Display for NewIndexError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Attempted to modify a read-only table.")
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn metatable_readonly() {
		let lua = Lua::new_ex(true, true).unwrap();
		let table = lua.create_table().unwrap();
		table.set_metatable(Some(lua.metatable_readonly()));
		lua.globals().set("test_table", table).unwrap();

		const CHUNK: &str = "test_table.field = 0";
		let c = lua.safeload(CHUNK, "test_chunk", lua.globals());

		let err = match c.eval::<()>() {
			Ok(()) => panic!("Assignment succeeded unexpectedly."),
			Err(err) => err,
		};

		let cause = match err {
			LuaError::CallbackError { cause, .. } => cause,
			other => panic!("Unexpected Lua error kind: {:#?}", other),
		};

		match cause.as_ref() {
			LuaError::ExternalError(err) => assert!(err.is::<NewIndexError>()),
			other => panic!("Unexpected Lua callback error cause: {:#?}", other),
		}
	}
}
