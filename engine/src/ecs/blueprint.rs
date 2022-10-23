//! A blueprint is a template used to instantiate entities, as well as to

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

use std::fmt;

use mlua::prelude::*;

use crate::lua::ImpureLua;

/// A template used to instantiate entities.
#[derive(Debug)]
pub struct Blueprint {
	pub id: String,
	/// Any Lua fields defined on the blueprint table that don't have meaning
	/// to the engine are kept in a table in the registry.
	/// The blueprint table that gets exposed to scripts is a merge between
	/// ECS component userdata and the aforementioned auxiliary table.
	pub lua: LuaRegistryKey,
}

impl Blueprint {
	pub fn new(lua: &Lua, table: LuaTable) -> Result<Self, Error> {
		let mut id = String::with_capacity(64);

		let aux = lua
			.create_table()
			.expect("Failed to create blueprint auxiliary table.");

		for pair in table.pairs::<LuaString, LuaValue>() {
			let (key, value) = match pair {
				Ok(tup) => tup,
				Err(_) => {
					return Err(Error::InvalidPair);
				}
			};

			let key = match key.to_str() {
				Ok(k) => k,
				Err(err) => {
					return Err(Error::InvalidKeyUtf8(err));
				}
			};

			match key {
				"id" => {
					id = Self::read_id(value)?;
				}
				_ => {
					aux.set(key, value)
						.expect("Failed to set a blueprint auxiliary value.");
				}
			}
		}

		Ok(Blueprint {
			id,
			lua: lua
				.create_registry_value(aux)
				.expect("Failed to create blueprint auxiliary table's registry entry."),
		})
	}
}

// Internal implementation details: construction from Lua tables.
impl Blueprint {
	fn read_id(value: LuaValue) -> Result<String, Error> {
		match value {
			LuaValue::String(id) => match id.to_str() {
				Ok(s) => {
					if s.chars().any(|c| !(c.is_alphanumeric() || c == '_')) {
						Err(Error::InvalidIdChars)
					} else {
						Ok(s.to_string())
					}
				}
				Err(err) => Err(Error::InvalidIdUtf8(err)),
			},
			_ => Err(Error::NonStringId),
		}
	}
}

impl<'lua> ToLua<'lua> for Blueprint {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
		let ret = lua
			.create_table()
			.expect("Failed to create a table during a blueprint's to-Lua conversion.");

		ret.set("id", self.id).unwrap();

		let aux: LuaTable = lua
			.registry_value(&self.lua)
			.expect("A blueprint's auxiliary table wasn't found for a to-Lua conversion.");

		for pair in aux.pairs::<LuaString, LuaValue>() {
			let (key, value) = pair.expect("A blueprint was built with an invalid auxiliary pair.");

			ret.set(key, value)
				.expect("Failed to set a blueprint table's auxiliary values.");
		}

		Ok(LuaValue::Table(ret))
	}
}

#[derive(Debug)]
pub enum Error {
	InvalidPair,
	InvalidKeyUtf8(LuaError),
	NonStringId,
	InvalidIdChars,
	InvalidIdUtf8(LuaError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidPair => {
				write!(f, "The given blueprint contains a non-string key.")
			}
			Self::InvalidKeyUtf8(err) => {
				write!(
					f,
					"The given blueprint contained a key with invalid UTF-8. Error: {}",
					err
				)
			}
			Self::NonStringId => {
				write!(
					f,
					"The given blueprint contains a non-string under the key `id`."
				)
			}
			Self::InvalidIdChars => {
				write!(
					f,
					"The given blueprint's ID string contains characters \
					that aren't alphanumeric or underscores."
				)
			}
			Error::InvalidIdUtf8(err) => {
				write!(
					f,
					"The given blueprint's ID string is invalid UTF-8. Error: {}",
					err
				)
			}
		}
	}
}
