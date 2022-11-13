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

use log::{debug, error, info, warn};
use mlua::prelude::*;
use nanorand::WyRand;
use parking_lot::{Mutex, RwLock};

use crate::{
	lua::TableExt,
	rng::{ImpureRng, RngCore},
	sim::PlaySim,
	vfs::VirtualFs,
};

use super::{Error, ImpureLua};

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

/// Delete certain global symbols which would be unsafe for use by modders.
/// Note that not everything meeting that criteria is deleted (e.g. `jit`, `setfenv`),
/// but user-facing environments are always guaranteed to be safe.
pub(super) fn g_std(globals: &LuaTable) -> LuaResult<()> {
	const DELETED_KEYS: [&str; 4] = [
		"io",
		// Free functions
		"collectgarbage",
		"module",
		"print",
	];

	for key in DELETED_KEYS {
		globals.set(key, LuaValue::Nil)?;
	}

	Ok(())
}

fn logger(lua: &Lua, args: LuaMultiValue, func_name: &'static str) -> Result<String, String> {
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
					"String-representation error in `{}` arguments: {}",
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

/// The returned table becomes `_G.log`, and is populated with functions
/// mapping to [`info`], [`warn`], [`error`], and [`debug`].
pub(super) fn g_log(lua: &Lua) -> LuaResult<LuaTable> {
	let ret = lua.create_table()?;

	ret.set(
		"info",
		lua.create_function(|lua, args: LuaMultiValue| {
			match logger(lua, args, "log") {
				Ok(s) => info!("{}", s),
				Err(s) => error!("{}", s),
			};

			Ok(())
		})?,
	)?;

	ret.set(
		"warn",
		lua.create_function(|lua, args: LuaMultiValue| {
			match logger(lua, args, "warn") {
				Ok(s) => warn!("{}", s),
				Err(s) => error!("{}", s),
			};

			Ok(())
		})?,
	)?;

	ret.set(
		"err",
		lua.create_function(|lua, args: LuaMultiValue| {
			match logger(lua, args, "err") {
				Ok(s) => error!("{}", s),
				Err(s) => error!("{}", s),
			};

			Ok(())
		})?,
	)?;

	ret.set(
		"debug",
		lua.create_function(|lua, args: LuaMultiValue| {
			if lua.is_devmode() {
				match logger(lua, args, "debug") {
					Ok(s) => debug!("{}", s),
					Err(s) => error!("{}", s),
				};
			}

			Ok(())
		})?,
	)?;

	ret.set_metatable(Some(lua.metatable_readonly()));

	Ok(ret)
}

/// The returned (read-only) table becomes `_G.impure` and is populated with
/// functions for the most basic engine functionality, like getting its version.
pub(super) fn g_impure(lua: &Lua) -> LuaResult<LuaTable> {
	let ret = lua.create_table()?;

	ret.set(
		"version",
		lua.create_function(|_, _: ()| {
			Ok((
				env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap(),
				env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap(),
				env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap(),
			))
		})?,
	)?;

	ret.set_metatable(Some(lua.metatable_readonly()));

	Ok(ret)
}

/// Loads several Lua modules (those expected to receive prolific use) from the
/// engine's package and makes them available globally for all other environments.
pub(super) fn g_prelude(lua: &Lua, vfs: &VirtualFs) -> LuaResult<()> {
	let globals = lua.globals();

	let utils = vfs
		.read_str("/impure/lua/utils.lua")
		.map_err(|err| LuaError::ExternalError(Arc::new(err)))?;
	let utils = lua.safeload(utils, "utils", globals.clone());
	let utils: LuaTable = utils.eval()?;

	for pair in utils.pairs::<LuaString, LuaValue>() {
		let (key, val) = pair?;
		debug_assert!(!globals.contains_key(key.clone()).unwrap());
		globals.set(key, val)?;
	}

	let array = vfs
		.read_str("/impure/lua/array.lua")
		.map_err(|err| LuaError::ExternalError(Arc::new(err)))?;
	let array = lua.safeload(array, "array", globals.clone());
	let array: LuaTable = array.eval()?;
	array.set_metatable(Some(lua.metatable_readonly()));
	globals.set("array", array)?;

	let map = vfs
		.read_str("/impure/lua/map.lua")
		.map_err(|err| LuaError::ExternalError(Arc::new(err)))?;
	let map = lua.safeload(map, "map", globals.clone());
	let map: LuaTable = map.eval()?;
	map.set_metatable(Some(lua.metatable_readonly()));
	globals.set("map", map)?;

	Ok(())
}

/// Deletes parts of the `os` standard library that would allow running arbitrary
/// OS processes, modifying the file system, altering the locale, or accessing
/// the OS' environment variables.
/// Sets `_G.os` to use the readonly metatable when done.
pub(super) fn g_os(lua: &Lua) -> LuaResult<()> {
	const DELETED_KEYS: [&str; 7] = [
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

	let g_os: LuaTable = lua.globals().get("os")?;

	for key in DELETED_KEYS {
		g_os.set(key, LuaValue::Nil)?;
	}

	g_os.set_metatable(Some(lua.metatable_readonly()));

	Ok(())
}

/// Deletes everything in the `package` standard library except `loaded`, since
/// Impure has an in-house system tied to the VFS to replace it. Sets `_G.package`
/// and `_G.package.loaded` to use the readonly metatable when done.
pub(super) fn g_package(lua: &Lua) -> LuaResult<()> {
	const DELETED_KEYS: [&str; 6] = ["cpath", "loaders", "loadlib", "path", "preload", "seeall"];

	let g_package: LuaTable = lua.globals().get("package")?;

	for key in DELETED_KEYS {
		g_package.set(key, LuaValue::Nil)?;
	}

	let metatable_readonly = lua.metatable_readonly();
	g_package
		.get::<_, LuaTable>("loaded")
		.unwrap()
		.set_metatable(Some(metatable_readonly.clone()));
	g_package.set_metatable(Some(metatable_readonly.clone()));

	Ok(())
}

/// Extends the `math` standard library with new functions and constants and
/// sets it to be read-only.
pub(super) fn g_math(lua: &Lua) -> LuaResult<()> {
	let g_math: LuaTable = lua.globals().get("math")?;

	// Constants ///////////////////////////////////////////////////////////////

	let huge = g_math.get::<_, LuaNumber>("huge")?;
	let neg_huge = lua.load("-math.huge").eval::<LuaNumber>()?;

	g_math.set("INF", huge)?;
	g_math.set("NEG_INF", neg_huge)?;
	g_math.set("EPSILON", f64::EPSILON)?;
	g_math.set("MAX", f64::MAX)?;
	g_math.set("MIN", f64::MIN)?;

	// Functions ///////////////////////////////////////////////////////////////

	g_math.set(
		"acosh",
		lua.create_function(|_, num: LuaNumber| Ok(num.acosh()))?,
	)?;

	g_math.set(
		"asinh",
		lua.create_function(|_, num: LuaNumber| Ok(num.asinh()))?,
	)?;

	g_math.set(
		"atanh",
		lua.create_function(|_, num: LuaNumber| Ok(num.atanh()))?,
	)?;

	g_math.set(
		"cbrt",
		lua.create_function(|_, num: LuaNumber| Ok(num.cbrt()))?,
	)?;

	g_math.set(
		"lerp",
		lua.create_function(|_, args: (LuaNumber, LuaNumber, LuaNumber)| {
			Ok(args.0 + (args.1 - args.0) * args.2)
		})?,
	)?;

	g_math.set(
		"logn",
		lua.create_function(|_, nums: (LuaNumber, LuaNumber)| Ok(nums.0.log(nums.1)))?,
	)?;

	g_math.set(
		"log2",
		lua.create_function(|_, num: LuaNumber| Ok(num.log2()))?,
	)?;

	g_math.set(
		"hypotenuse",
		lua.create_function(|_, nums: (LuaNumber, LuaNumber)| Ok(nums.0.hypot(nums.1)))?,
	)?;

	g_math.set_metatable(Some(lua.metatable_readonly()));

	Ok(())
}

pub(super) fn g_rng_client(lua: &Lua, rng: Arc<Mutex<RngCore<WyRand>>>) -> LuaResult<LuaTable> {
	fn resolve_prng<'rng>(
		val: LuaValue,
		rng: &'rng mut RngCore<WyRand>,
	) -> LuaResult<&'rng mut WyRand> {
		if let LuaNil = val {
			Ok(rng.get_anon())
		} else if let LuaValue::String(id) = val {
			let id = id.to_str()?;

			match rng.try_get(id) {
				Some(p) => Ok(p),
				None => Err(LuaError::ExternalError(Arc::new(Error::NonExistentPrng(
					id.to_string(),
				)))),
			}
		} else {
			Err(LuaError::FromLuaConversionError {
				from: val.type_name(),
				to: "string",
				message: Some(
					"To specify which named RNG to use, the given ID must be a string.".to_string(),
				),
			})
		}
	}

	let ret = lua.create_table()?;

	let rng_c = rng.clone();

	ret.set(
		"int",
		lua.create_function(move |_, args: (i64, i64, LuaValue)| {
			let mut rng = rng_c.lock();
			let prng = resolve_prng(args.2, &mut rng)?;
			Ok(prng.range_i64(args.0, args.1))
		})?,
	)?;

	let rng_c = rng.clone();

	ret.set(
		"float",
		lua.create_function(move |_, args: (f64, f64, LuaValue)| {
			let mut rng = rng_c.lock();
			let prng = resolve_prng(args.2, &mut rng)?;
			Ok(prng.range_f64(args.0, args.1))
		})?,
	)?;

	let rng_c = rng.clone();

	ret.set(
		"pick",
		lua.create_function(move |_, args: (LuaTable, LuaValue)| {
			if args.0.is_empty() {
				return Ok(LuaValue::Nil);
			}

			let mut rng = rng_c.lock();
			let prng = resolve_prng(args.1, &mut rng)?;
			args.0.get(prng.range_i64(1, args.0.len()?))
		})?,
	)?;

	ret.set(
		"coinflip",
		lua.create_function(move |_, id: LuaValue| {
			let mut rng = rng.lock();
			let prng = resolve_prng(id, &mut rng)?;
			Ok(prng.coin_flip())
		})?,
	)?;

	ret.set_metatable(Some(lua.metatable_readonly()));

	Ok(ret)
}

pub(super) fn g_rng_sim(lua: &Lua, sim: Arc<RwLock<PlaySim>>) -> LuaResult<LuaTable> {
	fn resolve_prng<'sim>(val: LuaValue, sim: &'sim mut PlaySim) -> LuaResult<&'sim mut WyRand> {
		if let LuaNil = val {
			Ok(sim.rng.get_anon())
		} else if let LuaValue::String(id) = val {
			let id = id.to_str()?;

			match sim.rng.try_get(id) {
				Some(p) => Ok(p),
				None => Err(LuaError::ExternalError(Arc::new(Error::NonExistentPrng(
					id.to_string(),
				)))),
			}
		} else {
			Err(LuaError::FromLuaConversionError {
				from: val.type_name(),
				to: "string",
				message: Some(
					"To specify which named RNG to use, the given ID must be a string.".to_string(),
				),
			})
		}
	}

	let ret = lua.create_table()?;

	let sim_c = sim.clone();

	ret.set(
		"int",
		lua.create_function(move |_, args: (i64, i64, LuaValue)| {
			let mut sim = sim_c.write();
			let prng = resolve_prng(args.2, &mut sim)?;
			let num = LuaValue::Integer(prng.range_i64(args.0, args.1));
			Ok(num)
		})?,
	)?;

	let sim_c = sim.clone();

	ret.set(
		"float",
		lua.create_function(move |_, args: (f64, f64, LuaValue)| {
			let mut sim = sim_c.write();
			let prng = resolve_prng(args.2, &mut sim)?;
			Ok(prng.range_f64(args.0, args.1))
		})?,
	)?;

	let sim_c = sim.clone();

	ret.set(
		"pick",
		lua.create_function(move |_, args: (LuaTable, LuaValue)| {
			if args.0.is_empty() {
				return Ok(LuaValue::Nil);
			}

			let mut sim = sim_c.write();
			let prng = resolve_prng(args.1, &mut sim)?;
			args.0.get(prng.range_i64(1, args.0.len()?))
		})?,
	)?;

	ret.set(
		"coinflip",
		lua.create_function(move |_, id: LuaValue| {
			let mut sim = sim.write();
			let prng = resolve_prng(id, &mut sim)?;
			Ok(prng.coin_flip())
		})?,
	)?;

	ret.set_metatable(Some(lua.metatable_readonly()));

	Ok(ret)
}

/// See [`g_debug`].
fn g_debug_full<'t>(lua: &Lua, debug: LuaTable<'t>) -> LuaResult<LuaTable<'t>> {
	debug.set("mem", lua.create_function(|l, ()| Ok(l.used_memory()))?)?;

	Ok(debug)
}

/// See [`g_debug`].
fn g_debug_noop(lua: &Lua) -> LuaResult<LuaTable> {
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

	Ok(ret)
}

/// When not running in "developer" mode (`-d` or `--dev`), the Lua state is
/// constructed without the `debug` stdlib. Replace it and its functions with
/// no-op versions under the same names so that these symbols can be used in
/// normal code, work normally when developing, but then be optimized away
/// when the end user runs that code.
pub(super) fn g_debug(lua: &Lua) -> LuaResult<LuaTable> {
	let debug = if let Ok(LuaValue::Table(debug)) = lua.globals().get("debug") {
		g_debug_full(lua, debug)?
	} else {
		debug_assert!(!lua.is_devmode()); // Minor sanity check
		g_debug_noop(lua)?
	};

	debug.set_metatable(Some(lua.metatable_readonly()));

	Ok(debug)
}

/// Returns a replacement for the standard global function `require`.
/// This in-house alternative acts as a convenient interface for the VFS.
pub(super) fn g_require(lua: &Lua, vfs: Arc<RwLock<VirtualFs>>) -> LuaResult<LuaFunction> {
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
	})
}

/// Returns a (read-only) table populated with functions for accessing the virtual file system.
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

	ret.set_metatable(Some(lua.metatable_readonly()));

	Ok(ret)
}
