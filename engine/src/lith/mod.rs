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

#[must_use]
#[allow(unused)]
pub(crate) fn is_keyword(string: &str) -> bool {
	const KEYWORDS: &[&str] = &[
		"abstract",
		"break",
		"catch",
		"ceval",
		"class",
		"const",
		"continue",
		"defer",
		"do",
		"else",
		"enum",
		"final",
		"finally",
		"for",
		"foreach",
		"if",
		"in",
		"interface",
		"is",
		"let",
		"loop",
		"macro",
		"mixin",
		"native",
		"out",
		"override",
		"private",
		"property",
		"protected",
		"public",
		"return",
		"static",
		"struct",
		"try",
		"type",
		"typeof",
		"union",
		"unsafe",
		"until",
		"virtual",
		"while",
		"yield",
	];

	KEYWORDS.iter().any(|s| *s == string)
}
