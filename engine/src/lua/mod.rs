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

use crate::{newtype, vfs::VirtualFs};
use log::{debug, error, info, warn};
use mlua::{prelude::*, TableExt as LuaTableExt};
use parking_lot::RwLock;
use std::{fmt, sync::Arc};

mod detail;

/// Only exists to extend [`mlua::Lua`] with new methods.
pub trait ImpureLua<'p> {
	/// Seeds the RNG, defines some dependency-free global functions (logging, etc.).
	/// If `safe` is `false`, the debug and FFI libraries are loaded.
	/// If `clientside` is `true`, the state's registry will contain the key-value
	/// pair `['clientside'] = true`. Otherwise, this key will be left nil.
	fn new_ex(safe: bool) -> LuaResult<Lua>;

	/// Modifies the Lua global environment to be more conducive to a safe,
	/// Impure-suitable sandbox, and adds numerous Impure-specific symbols.
	fn global_init(&self, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<()>;

	/// Adds `math`, `string`, and `table` standard libraries to an environment,
	/// as well as several standard free functions and `_VERSION`.
	fn envbuild_std(&self, env: &LuaTable);

	fn getfenv(&self) -> LuaTable;

	fn metatable_readonly(&self) -> LuaTable;

	fn set_clientside(&self, clientside: bool);

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
	fn new_ex(safe: bool) -> LuaResult<Lua> {
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

		ret.set_app_data(ClientsideAppData(true));

		detail::randomseed(&ret)?;

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

		impure.set(
			"log",
			ret.create_function(|lua, args: LuaMultiValue| {
				match detail::logger(lua, args, "log") {
					Ok(s) => info!("{}", s),
					Err(s) => error!("{}", s),
				};

				Ok(())
			})?,
		)?;

		impure.set(
			"warn",
			ret.create_function(|lua, args: LuaMultiValue| {
				match detail::logger(lua, args, "warn") {
					Ok(s) => warn!("{}", s),
					Err(s) => error!("{}", s),
				};

				Ok(())
			})?,
		)?;

		impure.set(
			"err",
			ret.create_function(|lua, args: LuaMultiValue| {
				match detail::logger(lua, args, "err") {
					Ok(s) => error!("{}", s),
					Err(s) => error!("{}", s),
				};

				Ok(())
			})?,
		)?;

		impure.set(
			"debug",
			ret.create_function(|lua, args: LuaMultiValue| {
				match detail::logger(lua, args, "debug") {
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
		let globals = self.globals();

		detail::delete_g(&globals)?;
		detail::delete_g_os(&globals)?;
		detail::delete_g_package(&globals)?;

		globals
			.get::<_, LuaTable>("package")?
			.get::<_, LuaTable>("loaded")?
			.set_metatable(Some(self.metatable_readonly()));

		detail::g_require(self, vfs.clone())?;
		detail::g_vfs(self, vfs.clone())?;

		globals.set("vfs", detail::g_vfs(self, vfs.clone())?)?;

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
			env.is_empty(),
			"`ImpureLua::env_init_std`: Called on a non-empty table."
		);

		let globals = self.globals();

		const GLOBAL_KEYS: [&str; 19] = [
			"_VERSION",
			// Tables
			"math",
			"os",
			"package",
			"string",
			"table",
			// Free functions
			"error",
			"getmetatable",
			"ipairs",
			"next",
			"pairs",
			"pcall",
			"require",
			"select",
			"tonumber",
			"tostring",
			"type",
			"unpack",
			"xpcall",
		];

		for key in GLOBAL_KEYS {
			let val = globals
				.get::<&str, LuaValue>(key)
				.expect("`ImpureLua::env_init_std`: global `{}` is missing.");

			env.set(key, val).unwrap_or_else(|err| {
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

	fn set_clientside(&self, clientside: bool) {
		self.app_data_mut::<ClientsideAppData>().unwrap().0 = clientside;
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

/// Only exists to extend [`mlua::Table`] with new methods.
pub trait TableExt<'lua> {
	/// Checks if a table has no key-value pairs or sequence pairs.
	/// Bypasses metamethods.
	fn is_empty(&self) -> bool;

	/// Returns the number of key-value pairs in this table.
	fn pair_len(&self) -> usize;
}

impl<'lua> TableExt<'_> for LuaTable<'lua> {
	fn is_empty(&self) -> bool {
		self.raw_len() == 0 && self.pair_len() == 0
	}

	fn pair_len(&self) -> usize {
		self.clone().pairs::<LuaString, LuaValue>().count()
	}
}

#[must_use]
pub fn is_reserved_keyword(string: &str) -> bool {
	#[rustfmt::skip]
	const RESERVED_KEYWORDS: &[&str] = &[
		"and",   
		"break",    
		"do",      
		"else",     
		"elseif",
		"end",    
		"false",   
		"for",     
		"function",  
		"if",
		"in",     
		"local",   
		"nil",     
		"not",       
		"or",
		"repeat", 
		"return",  
		"then",    
		"true",      
		"until",     
		"while",
	];

	RESERVED_KEYWORDS.iter().any(|s| s == &string)
}

newtype!(
	/// Unique type for use as Lua app data, indicating whether the owning state
	/// is currently in the process of running a sim tick.
	pub struct ClientsideAppData(bool)
);

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
		let lua = Lua::new_ex(true).unwrap();
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
