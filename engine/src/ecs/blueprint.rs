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
				Err(err) => {
					return Err(touchup_fromluaconv_error(
						err,
						"string",
						"Blueprint contains a non-string key.",
					));
				}
			};

			let key = key
				.to_str()
				.map_err(|err| map_lua_utf8_error(err, "Invalid characters in a `special` key:"))?;

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
		let string = if let LuaValue::String(s) = value {
			s
		} else {
			return Err(fromluaconv_error(
				"string",
				&value,
				"A blueprint's ID must be a string.",
			));
		};

		let id = string
			.to_str()
			.map_err(|err| map_lua_utf8_error(err, "A blueprint's ID must be valid UTF-8 text."))?;

		if id.chars().any(|c| !(c.is_alphanumeric() || c == '_')) {
			return Err(Error::MiscWithStr(
				r"
A blueprint's ID can only consist of letters, numbers, and underscores.
",
			));
		}

		Ok(id.to_string())
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

		ret.set_metatable(Some(lua.metatable_readonly()));

		Ok(LuaValue::Table(ret))
	}
}

#[derive(Debug)]
pub enum Error {
	/// Generally wraps a [`mlua::Error::FromLuaConversionError`].
	Lua(LuaError),
	MiscWithStr(&'static str),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Lua(err) => err.fmt(f),
			Self::MiscWithStr(string) => string.fmt(f),
		}
	}
}

/// Takes an [`mlua::Error`] emitted by [`mlua::String::to_str`], guaranteed to
/// be of the variant `FromLuaConversionError`. The same type and variant is emitted,
/// wrapped in a blueprint [`Error`], with `to` changed from `&str` to `UTF-8` for
/// the user's benefit. The message also gets context added.
fn map_lua_utf8_error(err: LuaError, context: &'static str) -> Error {
	if let LuaError::FromLuaConversionError { message, .. } = err {
		let message = message.expect(
			"`mlua::String::to_str` unexpectedly returned an error without an attached message.",
		);

		Error::Lua(LuaError::FromLuaConversionError {
			from: "string",
			to: "UTF-8",
			message: Some(format!("{context}: {message}")),
		})
	} else {
		unreachable!("`mlua::String::to_str` unexpectedly returned: {err}");
	}
}

fn fromluaconv_error(expected: &'static str, value: &LuaValue, message: &'static str) -> Error {
	Error::Lua(LuaError::FromLuaConversionError {
		from: value.type_name(),
		to: expected,
		message: Some(message.to_string()),
	})
}

fn touchup_fromluaconv_error(
	err: LuaError,
	expected: &'static str,
	context: &'static str,
) -> Error {
	if let LuaError::FromLuaConversionError { from, .. } = err {
		Error::Lua(LuaError::FromLuaConversionError {
			from,
			to: expected,
			message: Some(context.to_string()),
		})
	} else {
		unreachable!("`touchup_fromluaconv_error` unexpectedly received: {err}");
	}
}
