//! End-to-end transpilation from the ZDoom-family CVARINFO lump format into
//! a Lua table representing the `prefs` component of a package manifest.

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
	fmt,
	num::{ParseFloatError, ParseIntError},
	str::ParseBoolError,
};

use mlua::prelude::*;

#[derive(Debug)]
pub enum Error {
	Lua(LuaError),
	Empty,
	Unterminated,
	NoScope,
	InvalidScope(String),
	NoKind,
	NoId,
	InvalidId,
	KeywordId,
	DefaultParseBool(ParseBoolError),
	DefaultParseInt(ParseIntError),
	DefaultParseFloat(ParseFloatError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Lua(err) => {
				write!(f, "Lua error: {}", err)
			}
			Self::Empty => {
				write!(f, "CVar declaration has no content.")
			}
			Self::Unterminated => {
				write!(f, "All CVar declarations must end in a semicolon.")
			}
			Self::NoScope => {
				write!(f, "CVar declaration lacks a scope specifier.")
			}
			Self::InvalidScope(string) => {
				write!(
					f,
					"Found scope specifier: `{}`.
					Expected `server` or `user` or `nosave`.",
					string
				)
			}
			Self::NoKind => {
				write!(f, "No type specifier found in CVar declaration.")
			}
			Self::NoId => {
				write!(f, "No ID string was found in CVar declaration.")
			}
			Self::InvalidId => {
				write!(
					f,
					"ID can only contain ASCII alphanumeric characters and underscores."
				)
			}
			Self::KeywordId => {
				write!(
					f,
					"ID can not overlap with any of the following keywords:
					`server` `user` `nosave`
					`int` `float` `color` `bool` `string
					`noarchive` `cheat` `latch`"
				)
			}
			Self::DefaultParseBool(err) => {
				write!(f, "Failed to parse default boolean value: {}", err)
			}
			Self::DefaultParseInt(err) => {
				write!(f, "Failed to parse default integer value: {}", err)
			}
			Self::DefaultParseFloat(err) => {
				write!(f, "Failed to parse default floating-point value: {}", err)
			}
		}
	}
}

enum CVarKind {
	Bool,
	Int,
	Float,
	String,
	Color,
}

pub fn transpile<'lua>(lua: &'lua Lua, lump: &str) -> Result<LuaTable<'lua>, Error> {
	fn map_lua_err(err: LuaError) -> Error {
		Error::Lua(err)
	}

	fn is_valid_id(string: &str) -> bool {
		for c in string.chars() {
			if !(c.is_ascii_alphanumeric() || c == '_') {
				return false;
			}
		}

		true
	}

	fn is_keyword(string: &str) -> bool {
		const KEYWORDS: &[&str] = &[
			"server",
			"user",
			"nosave",
			"int",
			"float",
			"color",
			"bool",
			"string",
			"noarchive",
			"cheat",
			"latch",
		];

		for keyword in KEYWORDS {
			if string.eq_ignore_ascii_case(keyword) {
				return true;
			}
		}

		false
	}

	fn whitespace_only(string: &str) -> bool {
		for c in string.chars() {
			if !c.is_whitespace() {
				return false;
			}
		}

		true
	}

	let ret = lua.create_table().map_err(map_lua_err)?;

	for decl in lump.split_inclusive(';') {
		if whitespace_only(decl) {
			continue;
		}

		if !decl.ends_with(';') {
			return Err(Error::Unterminated);
		}

		let mut halves = decl.split('=');
		let lhs = halves.next().ok_or(Error::Empty)?.trim();

		// A token can be empty or just whitespace if the user chains spaces/tabs/etc.
		let mut tokens = lhs
			.split(char::is_whitespace)
			.filter(|t| !t.is_empty() && !whitespace_only(t));

		let scope = tokens.next().ok_or(Error::NoScope)?.trim();
		let server = scope.eq_ignore_ascii_case("server");
		let user = scope.eq_ignore_ascii_case("user");
		let mut nosave = scope.eq_ignore_ascii_case("nosave");

		if !server && !user && !nosave {
			return Err(Error::InvalidScope(scope.to_string()));
		}

		let tcount = tokens.clone().count();
		let id = tokens.clone().last().ok_or(Error::NoId)?.trim();

		if !is_valid_id(id) {
			return Err(Error::InvalidId);
		}

		if is_keyword(id) {
			return Err(Error::KeywordId);
		}

		let mut kind = None;
		let mut latch = false;
		let mut cheat = false;
		let mut noarchive = false;

		for (index, token) in tokens.clone().enumerate() {
			if index == (tcount - 2) {
				if token.eq_ignore_ascii_case("bool") {
					kind = Some(CVarKind::Bool);
				} else if token.eq_ignore_ascii_case("int") {
					kind = Some(CVarKind::Int);
				} else if token.eq_ignore_ascii_case("float") {
					kind = Some(CVarKind::Float);
				} else if token.eq_ignore_ascii_case("color") {
					kind = Some(CVarKind::Color);
				} else if token.eq_ignore_ascii_case("string") {
					kind = Some(CVarKind::String);
				}
			} else if token.eq_ignore_ascii_case("cheat") {
				cheat = true;
			} else if token.eq_ignore_ascii_case("latch") {
				latch = true;
			} else if token.eq_ignore_ascii_case("noarchive") {
				noarchive = true;
			} else if token.eq_ignore_ascii_case("nosave") {
				nosave = true;
			}
		}

		let kind = kind.ok_or(Error::NoKind)?;
		let table = lua.create_table().map_err(map_lua_err)?;

		if let Some(rhs) = halves.next() {
			let default = rhs.trim().trim_end_matches(';');

			let res = match kind {
				CVarKind::Bool => match default.parse::<bool>() {
					Ok(b) => table.set("default", b),
					Err(err) => return Err(Error::DefaultParseBool(err)),
				},
				CVarKind::Int => match default.parse::<i32>() {
					Ok(i) => table.set("default", i),
					Err(err) => return Err(Error::DefaultParseInt(err)),
				},
				CVarKind::Float => match default.parse::<f32>() {
					Ok(f) => table.set("default", f),
					Err(err) => return Err(Error::DefaultParseFloat(err)),
				},
				CVarKind::String | CVarKind::Color => {
					table.set("default", default.trim_matches('"'))
				}
			};

			res.map_err(map_lua_err)?;
		}

		table
			.set(
				"scope",
				if server {
					"server"
				} else if user || nosave {
					"client"
				} else {
					unreachable!()
				},
			)
			.map_err(map_lua_err)?;

		table
			.set(
				"kind",
				match kind {
					CVarKind::Bool => "bool",
					CVarKind::Int => "int",
					CVarKind::Float => "float",
					CVarKind::String => "string",
					CVarKind::Color => "color",
				},
			)
			.map_err(map_lua_err)?;

		table.set("saved", !nosave).map_err(map_lua_err)?;
		table.set("latch", latch).map_err(map_lua_err)?;
		table.set("cheat", cheat).map_err(map_lua_err)?;
		table.set("written", !noarchive).map_err(map_lua_err)?;

		ret.set(id, table).map_err(map_lua_err)?;
	}

	Ok(ret)
}

#[cfg(test)]
mod test {
	use crate::lua::ImpureLua;

	use super::*;

	fn scope_is_server(pref: &LuaTable) -> bool {
		pref.get::<&str, String>("scope").unwrap() == "server"
	}

	fn scope_is_client(pref: &LuaTable) -> bool {
		pref.get::<&str, String>("scope").unwrap() == "client"
	}

	fn is_of_kind(pref: &LuaTable, kind: &str) -> bool {
		pref.get::<&str, String>("kind").unwrap() == kind
	}

	fn is_saved(pref: &LuaTable) -> bool {
		pref.get::<&str, bool>("saved").unwrap() == true
	}

	fn is_written(pref: &LuaTable) -> bool {
		pref.get::<&str, bool>("written").unwrap() == true
	}

	fn is_cheat(pref: &LuaTable) -> bool {
		pref.get::<&str, bool>("cheat").unwrap() == true
	}

	fn is_latching(pref: &LuaTable) -> bool {
		pref.get::<&str, bool>("latch").unwrap() == true
	}

	#[test]
	fn valid() {
		const SOURCE: &str = r#"
server int delusive_bunker = 42;

USER
latch BOOL MEAT_GRINDER = false;

nOsAvE 	cheat color blueroom =
"F5 3A 95";

nosave server
noarchive 	float
fullConfession = 0.369;

user
	nosave
string	
KatanaZERO
=
"LudoWic";

		"#;

		let lua = Lua::new_ex(true).expect("Failed to create Lua state.");

		let table = match transpile(&lua, SOURCE) {
			Ok(t) => t,
			Err(err) => {
				panic!("{}", err);
			}
		};

		let pref1 = match table.get::<&str, LuaTable>("delusive_bunker") {
			Ok(p) => p,
			Err(err) => {
				panic!("Failed to retrieve pref 1: {}", err);
			}
		};

		assert_eq!(pref1.get::<&str, i32>("default").unwrap(), 42);
		assert!(is_of_kind(&pref1, "int"));
		assert!(scope_is_server(&pref1));
		assert!(is_saved(&pref1));

		let pref2 = match table.get::<&str, LuaTable>("MEAT_GRINDER") {
			Ok(p) => p,
			Err(err) => {
				panic!("Failed to retrieve pref 2: {}", err);
			}
		};

		assert!(is_latching(&pref2));

		let pref3 = match table.get::<&str, LuaTable>("blueroom") {
			Ok(p) => p,
			Err(err) => {
				panic!("Failed to retrieve pref 3: {}", err);
			}
		};

		assert!(is_cheat(&pref3));

		let pref4 = match table.get::<&str, LuaTable>("fullConfession") {
			Ok(p) => p,
			Err(err) => {
				panic!("Failed to retrieve pref 4: {}", err);
			}
		};

		assert!(!is_written(&pref4));

		let pref5 = match table.get::<&str, LuaTable>("KatanaZERO") {
			Ok(p) => p,
			Err(err) => {
				panic!("Failed to retrieve pref 5: {}", err);
			}
		};

		assert_eq!(pref5.get::<&str, String>("default").unwrap(), "LudoWic");
		assert_eq!(pref5.get::<&str, String>("kind").unwrap(), "string");
		assert!(scope_is_client(&pref5));
		assert!(!is_saved(&pref5));
	}
}
