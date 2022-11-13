//! Impure-specific Lua behaviour and myriad utilities.

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

use std::{fmt, sync::Arc};

use mlua::{prelude::*, TableExt as LuaTableExt};
use nanorand::WyRand;
use parking_lot::{Mutex, RwLock};

use crate::{newtype, rng::RngCore, sim::PlaySim, vfs::VirtualFs};

mod detail;
mod vector;

/// Only exists to extend [`mlua::Lua`] with new methods.
pub trait ImpureLua<'p> {
	/// Seeds the RNG, defines some universal app data and registry values.
	/// If `safe` is `false`, the debug and FFI libraries are loaded.
	fn new_ex(safe: bool) -> LuaResult<Lua>;

	/// Modifies the Lua global environment to be more conducive to a safe,
	/// Impure-suitable sandbox, and adds numerous Impure-specific symbols.
	fn init_api_common(&self, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<()>;

	/// Loads the registry with tables (to be loaded into `_G` whenever a sim tic
	/// ends) containing functions that access and modify the client state.
	fn init_api_client(&self, rng: Option<Arc<Mutex<RngCore<WyRand>>>>) -> LuaResult<()>;
	/// Loads `_G` with tables containing functions that access and modify
	/// the client state. Call when a sim tic ends, or building client state.
	fn load_api_client(&self);

	/// Loads the registry with tables (to be loaded into `_G` whenever a sim tic
	/// starts) containing functions that access and modify the sim state.
	fn init_api_playsim(&self, sim: Arc<RwLock<PlaySim>>) -> LuaResult<()>;
	/// Loads `_G` with tables containing functions that access and modify
	/// the sim state. Call when a sim tic starts.
	fn load_api_playsim(&self);
	fn clear_api_playsim(&self) -> LuaResult<()>;

	/// Adds `math`, `string`, and `table` standard libraries to an environment,
	/// as well as several standard free functions and `_VERSION`.
	fn envbuild_std(&self, env: &LuaTable);

	fn getfenv(&self) -> LuaTable;

	fn metatable_readonly(&self) -> LuaTable;

	fn start_sim_tic(&self);
	fn finish_sim_tic(&self);

	#[must_use]
	fn is_devmode(&self) -> bool;

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

const REGID_CLIENT_RNG: &str = "api/client/rng";
const REGID_SIM_RNG: &str = "api/sim/rng";

impl<'p> ImpureLua<'p> for mlua::Lua {
	fn new_ex(safe: bool) -> LuaResult<Lua> {
		// Note: `io`, `os`, and `package` aren't sandbox-safe by themselves.
		// They either get pruned of dangerous functions by global API init
		// functions or are deleted now and may get returned in reduced form
		// in the future.

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

		let ret = if safe {
			Lua::new_with(safe_libs, LuaOptions::default())?
		} else {
			unsafe {
				Lua::unsafe_new_with(
					safe_libs | LuaStdLib::DEBUG | LuaStdLib::FFI,
					LuaOptions::default(),
				)
			}
		};

		ret.set_app_data(DevModeAppData(!safe));
		ret.set_app_data(SimsideAppData(true));

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
					Err(LuaError::ExternalError(Arc::new(Error::IllegalNewIndex)))
				})
				.expect("Failed to create `__newindex` function for `metas.readonly`."),
			)
			.expect("Failed to set function `metas.readonly.__newindex`.");

		metas
			.set("readonly", readonly)
			.expect("Failed to set `metas.readonly`.");

		ret.set_named_registry_value("metas", metas)
			.expect("Failed to set table `metas` in registry.");

		Ok(ret)
	}

	fn init_api_common(&self, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<()> {
		let globals = self.globals();

		detail::g_std(&globals)?;
		detail::g_math(self)?;
		detail::g_vector(self)?;
		detail::g_os(self)?;
		detail::g_package(self)?;
		globals.set("impure", detail::g_impure(self)?)?;
		globals.set("log", detail::g_log(self)?)?;
		globals.set("debug", detail::g_debug(self)?)?;
		globals.set("require", detail::g_require(self, vfs.clone())?)?;
		globals.set("vfs", detail::g_vfs(self, vfs.clone())?)?;

		detail::g_prelude(self, &vfs.read())?;

		Ok(())
	}

	fn init_api_client(&self, rng: Option<Arc<Mutex<RngCore<WyRand>>>>) -> LuaResult<()> {
		if let Some(rng) = rng {
			self.set_named_registry_value(REGID_CLIENT_RNG, detail::g_rng_client(self, rng)?)?;
		}

		Ok(())
	}

	fn load_api_client(&self) {
		let globals = self.globals();

		globals
			.set(
				"rng",
				self.named_registry_value::<_, LuaTable>(REGID_CLIENT_RNG)
					.unwrap(),
			)
			.unwrap();
	}

	fn init_api_playsim(&self, sim: Arc<RwLock<PlaySim>>) -> LuaResult<()> {
		self.set_named_registry_value(REGID_SIM_RNG, detail::g_rng_sim(self, sim)?)?;

		Ok(())
	}

	fn load_api_playsim(&self) {
		let globals = self.globals();

		globals
			.set(
				"rng",
				self.named_registry_value::<_, LuaTable>(REGID_SIM_RNG)
					.unwrap(),
			)
			.unwrap();
	}

	fn clear_api_playsim(&self) -> LuaResult<()> {
		self.unset_named_registry_value(REGID_SIM_RNG)
	}

	fn envbuild_std(&self, env: &LuaTable) {
		debug_assert!(
			env.is_empty(),
			"Lua env-init (std.): Called on a non-empty table."
		);

		let globals = self.globals();

		const GLOBAL_KEYS: [&str; 20] = [
			"_VERSION",
			// Tables
			"debug",
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
				.expect("Lua env-init (std.): Cglobal `{}` is missing.");

			env.set(key, val).unwrap_or_else(|err| {
				panic!("Lua env-init (std.): Cfailed to set `{}` ({}).", key, err)
			});
		}

		let debug: LuaResult<LuaTable> = globals.get("debug");

		if let Ok(d) = debug {
			env.set("debug", d)
				.expect("Lua env-init (std.): CFailed to set `debug`.");
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

	fn start_sim_tic(&self) {
		self.app_data_mut::<SimsideAppData>().unwrap().0 = true;
		self.load_api_playsim();
	}

	fn finish_sim_tic(&self) {
		self.app_data_mut::<SimsideAppData>().unwrap().0 = false;
		self.load_api_client();
	}

	#[must_use]
	fn is_devmode(&self) -> bool {
		**self.app_data_ref::<DevModeAppData>().unwrap()
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

/// [`mlua::UserData`] can't be implemented for external types,
/// so wrap them in this and then provide that implementation.
#[derive(Clone)]
pub struct UserDataWrapper<T: Sized + Clone>(pub T);

impl<T: Sized + Clone> std::ops::Deref for UserDataWrapper<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Sized + Clone> std::ops::DerefMut for UserDataWrapper<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub type Vec2 = UserDataWrapper<glam::Vec2>;
pub type Vec3 = UserDataWrapper<glam::Vec3>;
pub type Vec4 = UserDataWrapper<glam::Vec4>;

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
	/// is currently in the process of running a sim tic.
	pub struct SimsideAppData(bool)
);

newtype!(
	/// Unique type for use as Lua app data, indicating whether the owning state
	/// was initialized with launch arguments `-d` or `--dev`.
	pub struct DevModeAppData(bool)
);

#[derive(Debug)]
pub enum Error {
	IllegalNewIndex,
	NonExistentPrng(String),
	IndexOutOfRange {
		given: usize,
		min: usize,
		max: usize,
	},
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::IllegalNewIndex => {
				write!(f, "Attempted to modify a read-only table.")
			}
			Self::NonExistentPrng(id) => {
				write!(f, "No random number generator under the ID: {}", id)
			}
			Self::IndexOutOfRange { given, min, max } => {
				write!(
					f,
					"Expected index between {min} and {max} (inclusive); got {given}."
				)
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn metatable_readonly() {
		let lua = Lua::new_ex(true).unwrap();
		let table = lua.create_table().unwrap();
		table.set("field2", 23).unwrap();
		table.set_metatable(Some(lua.metatable_readonly()));
		lua.globals().set("test_table", table).unwrap();

		const CHUNK_NEWINDEX: &str = "test_table.field1 = 0";
		let c = lua.safeload(CHUNK_NEWINDEX, "test_chunk", lua.globals());

		let err = match c.eval::<()>() {
			Ok(()) => panic!("Assignment succeeded unexpectedly."),
			Err(err) => err,
		};

		let cause = match err {
			LuaError::CallbackError { cause, .. } => cause,
			other => panic!("Unexpected Lua error kind: {:#?}", other),
		};

		match cause.as_ref() {
			LuaError::ExternalError(err) => assert!(err.is::<Error>()),
			other => panic!("Unexpected Lua callback error cause: {:#?}", other),
		}

		const CHUNK_INDEX: &str = "return test_table.field2";
		let c = lua.safeload(CHUNK_INDEX, "test_chunk", lua.globals());

		match c.eval::<LuaValue>() {
			Ok(LuaValue::Integer(num)) => assert_eq!(num, 23),
			Ok(other) => panic!("Expected integer, got: {:#?}", other),
			Err(err) => panic!("{}", err),
		};
	}
}
