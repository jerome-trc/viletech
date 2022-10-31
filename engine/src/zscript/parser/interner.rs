/*

Copyright (C) 2021-2022 Jessica "Gutawer" Russell

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

use std::collections::HashMap;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Symbol<const CASE_SENSITIVE: bool>(usize);

#[derive(Debug, Clone, Default)]
pub struct Interner<const CASE_SENSITIVE: bool> {
	symbol_map: HashMap<Box<str>, Symbol<CASE_SENSITIVE>>,
	string_map: Vec<Box<str>>,
}

impl<const CASE_SENSITIVE: bool> std::fmt::Display for Interner<CASE_SENSITIVE> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(f, "{{")?;
		for (i, s) in self.string_map.iter().enumerate() {
			writeln!(f, "    {} => {:?},", i, s)?;
		}
		write!(f, "}}")?;
		Ok(())
	}
}

impl<const CASE_SENSITIVE: bool> Interner<CASE_SENSITIVE> {
	pub fn interned(&mut self, string: &str) -> Symbol<CASE_SENSITIVE> {
		if CASE_SENSITIVE {
			if let Some(s) = self.symbol_map.get(string) {
				return *s;
			}
		} else if let Some(s) = self.symbol_map.get(&*string.to_lowercase()) {
			return *s;
		}

		let new_sym = Symbol(self.string_map.len());
		if CASE_SENSITIVE {
			self.symbol_map
				.insert(string.to_string().into_boxed_str(), new_sym);
		} else {
			self.symbol_map
				.insert(string.to_lowercase().into_boxed_str(), new_sym);
		}
		self.string_map.push(string.to_string().into_boxed_str());

		new_sym
	}

	pub fn try_interned(&self, string: &str) -> Option<Symbol<CASE_SENSITIVE>> {
		if CASE_SENSITIVE {
			if let Some(s) = self.symbol_map.get(string) {
				return Some(*s);
			}
		} else if let Some(s) = self.symbol_map.get(&*string.to_lowercase()) {
			return Some(*s);
		}
		None
	}

	pub fn string(&self, symbol: Symbol<CASE_SENSITIVE>) -> &str {
		&self.string_map[symbol.0]
	}
}

static NAME_INTERNER: Lazy<RwLock<NameInterner>> =
	Lazy::new(|| RwLock::new(NameInterner::default()));

static STRING_INTERNER: Lazy<RwLock<StringInterner>> =
	Lazy::new(|| RwLock::new(StringInterner::default()));

pub fn intern_name(s: &str) -> NameSymbol {
	// try to get the interned symbol without having to mutably lock,
	// so that threads will only block others when they see a new symbol
	{
		let l = NAME_INTERNER.read();
		if let Some(s) = l.try_interned(s) {
			return s;
		}
	}
	{
		let mut l = NAME_INTERNER.write();
		l.interned(s)
	}
}

pub fn intern_string(s: &str) -> StringSymbol {
	// try to get the interned symbol without having to mutably lock,
	// so that threads will only block others when they see a new symbol
	{
		let l = STRING_INTERNER.read();
		if let Some(s) = l.try_interned(s) {
			return s;
		}
	}
	{
		let mut l = STRING_INTERNER.write();
		l.interned(s)
	}
}

pub type NameInterner = Interner<false>;
pub type NameSymbol = Symbol<false>;

impl std::fmt::Display for NameSymbol {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", NAME_INTERNER.read().string(*self))
	}
}

impl NameSymbol {
	pub fn string(&self) -> impl std::ops::Deref<Target = str> + 'static {
		parking_lot::RwLockReadGuard::map(NAME_INTERNER.read(), |s| s.string(*self))
	}
}

impl Serialize for NameSymbol {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(NAME_INTERNER.read().string(*self))
	}
}

pub type StringInterner = Interner<true>;
pub type StringSymbol = Symbol<true>;

impl std::fmt::Display for StringSymbol {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", STRING_INTERNER.read().string(*self))
	}
}

impl StringSymbol {
	pub fn string(&self) -> impl std::ops::Deref<Target = str> + 'static {
		parking_lot::RwLockReadGuard::map(STRING_INTERNER.read(), |s| s.string(*self))
	}
}

impl Serialize for StringSymbol {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(STRING_INTERNER.read().string(*self))
	}
}
