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

use super::data::*;
use crate::vfs::VirtualFs;
use log::{debug, error, info, warn};
use mlua::prelude::*;
use parking_lot::RwLock;
use std::{
	fs,
	path::{Path, PathBuf},
	sync::Arc,
	time::{SystemTime, UNIX_EPOCH},
};

pub trait ImpureLua<'p> {
	fn new_ex(safe: bool, vfs: Arc<RwLock<VirtualFs>>) -> Result<Lua, mlua::Error>;
	fn parse_package_meta(&self, path: &'p Path) -> Result<PkgMeta, PkgMetaParseError<'p>>;
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
				Ok(_) => {}
				Err(err) => warn!("Failed to seed a Lua state's RNG: {}", err),
			};
		}

		// Load the Teal compiler into the registry

		let teal: LuaTable = match ret.load(include_str!("./teal.lua")).eval() {
			Ok(t) => t,
			Err(err) => {
				return Err(err);
			}
		};

		match ret.set_named_registry_value("teal", teal) {
			Ok(_) => {}
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

				let bytes = match vfsg.read_bytes(path) {
					Ok(b) => b,
					Err(err) => {
						return Err(LuaError::ExternalError(Arc::new(err)));
					}
				};

				return l.load(&bytes).eval();
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

	fn parse_package_meta(&self, path: &'p Path) -> Result<PkgMeta, PkgMetaParseError<'p>> {
		let bytes = match fs::read(path) {
			Ok(b) => b,
			Err(err) => {
				return Err(PkgMetaParseError::FileReadError(path, err));
			}
		};

		let table: LuaTable = match self.load(&bytes).eval() {
			Ok(t) => t,
			Err(err) => {
				return Err(PkgMetaParseError::LuaEvalError(path, err));
			}
		};

		let verstable: LuaTable = match table.get("version") {
			Ok(t) => t,
			Err(err) => {
				return Err(PkgMetaParseError::LuaEvalError(path, err));
			}
		};

		let bmpstr = path.to_str().unwrap_or("<invalid path>");

		Ok(PkgMeta {
			uuid: table.get::<_, String>("uuid").unwrap_or_default(),
			version: SemVer {
				major: verstable.get::<_, u16>("major").unwrap_or_default(),
				minor: verstable.get::<_, u16>("minor").unwrap_or_default(),
				patch: verstable.get::<_, u16>("patch").unwrap_or_default(),
			},
			name: table
				.get::<_, String>("name")
				.unwrap_or_else(|_| "belltower.lang.pkgmeta_noname".to_string()),
			desc: table
				.get::<_, String>("description")
				.unwrap_or_else(|_| "belltower.lang.pkgmeta_nodesc".to_string()),
			author: table
				.get::<_, String>("author")
				.unwrap_or_else(|_| "belltower.lang.pkgmeta_noauthor".to_string()),
			copyright: table
				.get::<_, String>("copyright")
				.unwrap_or_else(|_| "belltower.lang.pkgmeta_nocopyright".to_string()),
			link: table.get::<_, String>("link").unwrap_or_default(),
			directory: PathBuf::from("data"),
			mount_point: table.get::<_, String>("uuid").unwrap_or_default(),
			dependencies: parse_dependencies(&table, bmpstr),
			incompatibilities: parse_incompatibilities(&table, bmpstr),
		})
	}
}
