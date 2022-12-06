//! Infrastructure powering the LithScript language.

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

#[allow(dead_code)]
pub mod ast;
pub mod parse;

pub(crate) mod keyword {
	pub const ABSTRACT: &str = "abstract";
	pub const BREAK: &str = "break";
	pub const CATCH: &str = "catch";
	pub const CEVAL: &str = "ceval";
	pub const CLASS: &str = "class";
	pub const CONST: &str = "const";
	pub const CONTINUE: &str = "continue";
	pub const DEFER: &str = "defer";
	pub const DO: &str = "do";
	pub const ELSE: &str = "else";
	pub const ENUM: &str = "enum";
	pub const FINAL: &str = "final";
	pub const FINALLY: &str = "finally";
	pub const FOR: &str = "for";
	pub const FOREACH: &str = "foreach";
	pub const IF: &str = "if";
	pub const IN: &str = "in";
	pub const INTERFACE: &str = "interface";
	pub const IS: &str = "is";
	pub const LET: &str = "let";
	pub const LOOP: &str = "loop";
	pub const MACRO: &str = "macro";
	pub const MIXIN: &str = "mixin";
	pub const NATIVE: &str = "native";
	pub const OUT: &str = "out";
	pub const OVERRIDE: &str = "override";
	pub const PRIVATE: &str = "private";
	pub const PROPERTY: &str = "property";
	pub const PROTECTED: &str = "protected";
	pub const PUBLIC: &str = "public";
	pub const RETURN: &str = "return";
	pub const STATIC: &str = "static";
	pub const STRUCT: &str = "struct";
	pub const TRY: &str = "try";
	pub const TYPE: &str = "type";
	pub const TYPEOF: &str = "typeof";
	pub const UNION: &str = "union";
	pub const UNSAFE: &str = "unsafe";
	pub const UNTIL: &str = "until";
	pub const VIRTUAL: &str = "virtual";
	pub const WHILE: &str = "while";
	pub const YIELD: &str = "yield";
}

#[must_use]
#[allow(unused)]
pub(crate) fn is_keyword(string: &str) -> bool {
	[
		keyword::ABSTRACT,
		keyword::BREAK,
		keyword::CATCH,
		keyword::CEVAL,
		keyword::CLASS,
		keyword::CONST,
		keyword::CONTINUE,
		keyword::DEFER,
		keyword::DO,
		keyword::ELSE,
		keyword::ENUM,
		keyword::FINAL,
		keyword::FINALLY,
		keyword::FOR,
		keyword::FOREACH,
		keyword::IF,
		keyword::IN,
		keyword::INTERFACE,
		keyword::IS,
		keyword::LET,
		keyword::LOOP,
		keyword::MACRO,
		keyword::MIXIN,
		keyword::NATIVE,
		keyword::OUT,
		keyword::OVERRIDE,
		keyword::PRIVATE,
		keyword::PROPERTY,
		keyword::PROTECTED,
		keyword::PUBLIC,
		keyword::RETURN,
		keyword::STATIC,
		keyword::STRUCT,
		keyword::TRY,
		keyword::TYPE,
		keyword::TYPEOF,
		keyword::UNION,
		keyword::UNSAFE,
		keyword::UNTIL,
		keyword::VIRTUAL,
		keyword::WHILE,
		keyword::YIELD,
	]
	.iter()
	.any(|s| *s == string)
}
