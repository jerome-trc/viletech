//! Implementation details for [`super::ImpureLua`].
//!
//! Kept out of mod.rs to reduce clutter.

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

use std::{
	sync::Arc,
	time::{SystemTime, UNIX_EPOCH},
};

use log::{error, warn};
use mlua::prelude::*;
use parking_lot::RwLock;

use crate::vfs::VirtualFs;

use super::ImpureLua;

/// Seed a Lua state's PRNG for trivial (i.e. client-side) purposes.
pub(super) fn randomseed(lua: &Lua) -> LuaResult<()> {
	let rseed: LuaFunction = lua
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

	Ok(())
}

pub(super) fn logger(
	lua: &Lua,
	args: LuaMultiValue,
	func_name: &'static str,
) -> Result<String, String> {
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
				"Invalid template string given to `{}`.\r\n\tError: {}",
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

/// Delete certain global symbols which would be unsafe for use by modders.
/// Note that not everything meeting that criteria is deleted (e.g. `jit`, `setfenv`),
/// but user-facing environments are always guaranteed to be safe.
pub(super) fn delete_g(globals: &LuaTable) -> LuaResult<()> {
	const KEYS_STD_GLOBAL: [&str; 4] = [
		"io",
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

/// Delete parts of the `os` standard library that would allow running arbitrary
/// OS processes, modifying the file system, altering the locale, or accessing
/// the OS' environment variables.
pub(super) fn delete_g_os(globals: &LuaTable) -> LuaResult<()> {
	const KEYS_STD_OS: [&str; 7] = [
		"execute",
		"exit",
		"getenv",
		"remove",
		"rename",
		"setlocale",
		"tmpname",
	];

	// Remaining symbols:
	// - `clock`
	// - `date`
	// - `difftime`
	// - `time`

	let g_os: LuaTable = globals.get("os")?;

	for key in KEYS_STD_OS {
		g_os.set(key, LuaValue::Nil)?;
	}

	Ok(())
}

/// Delete everything in the `package` standard library except `loaded`,
/// since Impure has an in-house system tied to the VFS to replace it.
pub(super) fn delete_g_package(globals: &LuaTable) -> LuaResult<()> {
	const KEYS_STD_PACKAGE: [&str; 6] =
		["cpath", "loaders", "loadlib", "path", "preload", "seeall"];

	let g_package: LuaTable = globals.get("package")?;

	for key in KEYS_STD_PACKAGE {
		g_package.set(key, LuaValue::Nil)?;
	}

	Ok(())
}

/// When not running in "developer" mode (`-d` or `--dev`), the Lua state is
/// constructed without the `debug` stdlib. Replace it and its functions with
/// no-op versions under the same names so that these symbols can be used in
/// normal code, work normally when developing, but then be optimized away
/// when the end user runs that code.
pub(super) fn g_debug_noop(lua: &Lua) -> LuaResult<LuaTable> {
	let ret = lua.create_table()?;

	// (Rat): I sure hope LuaJIT actually eliminates these calls...

	const KEYS: [&str; 15] = [
		"debug",
		"getfenv",
		"gethook",
		"getinfo",
		"getlocal",
		"getmetatable",
		"getregistry",
		"getupvalue",
		"setfenv",
		"sethook",
		"setlocal",
		"setmetatable",
		"setupvalue",
		"traceback",
		// Non-standard
		"mem",
	];

	let func = lua.create_function(|_, ()| Ok(()))?;

	for key in KEYS {
		ret.set(key, func.clone())?;
	}

	ret.set_metatable(Some(lua.metatable_readonly()));

	Ok(ret)
}

/// Replaces the standard global function `require` with an alternative
/// tied to the VFS.
pub(super) fn g_require(lua: &Lua, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<()> {
	lua.globals().set(
		"require",
		lua.create_function(move |l, path: String| -> LuaResult<LuaValue> {
			let loaded = l
				.globals()
				.raw_get::<_, LuaTable>("package")?
				.raw_get::<_, LuaTable>("loaded")?;

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
			let mut vpath = String::with_capacity(64);

			if !path.starts_with('/') {
				vpath.push('/');
			}

			for comp in path.split('.') {
				vpath.push_str(comp);
				vpath.push('/');
			}

			vpath.pop();
			vpath.push_str(".lua");

			let bytes = match vfs.read(&vpath) {
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

			let output = match l
				.safeload(chunk, path.as_str(), l.getfenv())
				.eval::<LuaValue>()
			{
				Ok(ret) => ret,
				Err(err) => {
					error!("{}", err);
					return Err(err);
				}
			};

			match output {
				LuaNil => {
					loaded.raw_set::<&str, bool>(&path, true)?;
					Ok(LuaValue::Boolean(true))
				}
				other => {
					loaded.raw_set::<&str, LuaValue>(&path, other.clone())?;
					Ok(other)
				}
			}
		})?,
	)
}

/// Takes in an empty table and outputs the same table, but with functions
/// for accessing the virtual file system.
pub(super) fn g_vfs(lua: &Lua, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<LuaTable> {
	let ret = lua.create_table()?;

	let read = lua.create_function(move |l, path: String| {
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

	ret.set("read", read)?;

	Ok(ret)
}
